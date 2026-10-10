use super::protocol::{
    LoginCommand, LoginData, connect_with_retries, parse_event, transport_error, validate_endpoint,
};
use super::{CloseInfo, WebSocketConfig, WebSocketError, WebSocketEvent};
use crate::credentials::redact;
use futures_util::{SinkExt, StreamExt};
use std::{collections::VecDeque, fmt};
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite};
use url::Url;

pub(super) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(super) struct Connection {
    socket: Socket,
    endpoint: String,
    api_key: String,
    config: WebSocketConfig,
    pending: VecDeque<WebSocketEvent>,
    closed: bool,
}

impl fmt::Debug for Connection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Connection")
            .field("endpoint", &self.endpoint)
            .field("api_key", &"[REDACTED]")
            .field("closed", &self.closed)
            .finish_non_exhaustive()
    }
}

impl Connection {
    pub(super) async fn connect(
        api_key: &str,
        endpoint: Url,
        config: WebSocketConfig,
    ) -> Result<Self, WebSocketError> {
        validate_endpoint(api_key, &endpoint)?;
        let endpoint = endpoint.to_string();
        let socket = connect_with_retries(&endpoint, &config, api_key).await?;
        let mut client = Self {
            socket,
            endpoint: redact(&endpoint, api_key),
            api_key: api_key.to_owned(),
            config,
            pending: VecDeque::new(),
            closed: false,
        };
        let login = serde_json::to_string(&LoginCommand {
            event: "login",
            data: LoginData { api_key },
        })
        .map_err(|error| client.protocol_error(&error.to_string(), "login serialization"))?;
        client.send_text(login, "login").await?;
        timeout(
            client.config.handshake_timeout,
            client.confirm_authentication(),
        )
        .await
        .map_err(|_| client.timeout_error("login confirmation"))??;
        Ok(client)
    }

    /// Receives a validated event within the configured receive timeout.
    ///
    /// Ping frames are answered and flushed automatically. Controls received
    /// during authentication are retained, including the login acknowledgment.
    pub(super) async fn next_event(&mut self) -> Result<WebSocketEvent, WebSocketError> {
        if let Some(event) = self.pending.pop_front() {
            return Ok(event);
        }
        timeout(self.config.receive_timeout, self.receive_event())
            .await
            .map_err(|_| self.timeout_error("receive"))?
    }

    /// Completes a close handshake within the configured handshake timeout.
    pub(super) async fn close(mut self) -> Result<(), WebSocketError> {
        if self.closed {
            return Ok(());
        }
        timeout(self.config.handshake_timeout, self.socket.close(None))
            .await
            .map_err(|_| self.timeout_error("close send"))?
            .map_err(|error| self.transport_error("close send", error))?;
        timeout(self.config.handshake_timeout, async {
            loop {
                if matches!(self.receive_event().await?, WebSocketEvent::Closed(_)) {
                    return Ok(());
                }
            }
        })
        .await
        .map_err(|_| self.timeout_error("close confirmation"))?
    }

    async fn confirm_authentication(&mut self) -> Result<(), WebSocketError> {
        loop {
            match self.receive_event().await? {
                WebSocketEvent::Control(control) => {
                    let confirmed = (self.config.confirm_login)(&control)
                        .map_err(|error| self.protocol_error(&error.to_string(), &control.json))?;
                    self.pending.push_back(WebSocketEvent::Control(control));
                    if confirmed {
                        return Ok(());
                    }
                    if self.pending.len() >= 32 {
                        return Err(self.protocol_error(
                            "received 32 controls without a positive login acknowledgment",
                            "",
                        ));
                    }
                }
                event @ (WebSocketEvent::Pong(_) | WebSocketEvent::Heartbeat(_)) => {
                    self.pending.push_back(event);
                    if self.pending.len() >= 32 {
                        return Err(self.protocol_error(
                            "received 32 events without a positive login acknowledgment",
                            "",
                        ));
                    }
                }
                WebSocketEvent::Closed(close) => {
                    return Err(self.protocol_error(
                        "peer closed before confirming authentication",
                        &close.reason,
                    ));
                }
                WebSocketEvent::Market(_) | WebSocketEvent::StreamQuote(_) => {
                    return Err(self.protocol_error(
                        "market data arrived before a positive login acknowledgment",
                        "",
                    ));
                }
            }
        }
    }

    pub(super) async fn send_text(
        &mut self,
        json: String,
        operation: &'static str,
    ) -> Result<(), WebSocketError> {
        if self.closed {
            return Err(WebSocketError::Closed {
                endpoint: self.endpoint.clone(),
            });
        }
        timeout(
            self.config.handshake_timeout,
            self.socket.send(tungstenite::Message::Text(json.into())),
        )
        .await
        .map_err(|_| self.timeout_error(operation))?
        .map_err(|error| self.transport_error(operation, error))
    }

    async fn receive_event(&mut self) -> Result<WebSocketEvent, WebSocketError> {
        if self.closed {
            return Err(WebSocketError::Closed {
                endpoint: self.endpoint.clone(),
            });
        }
        loop {
            let frame = self
                .socket
                .next()
                .await
                .ok_or_else(|| self.protocol_error("stream ended without a close frame", ""))?
                .map_err(|error| self.transport_error("receive", error))?;
            match frame {
                tungstenite::Message::Text(json) => {
                    return parse_event(&self.endpoint, &self.api_key, &json);
                }
                tungstenite::Message::Ping(_) => {
                    // Tungstenite queues the matching pong while reading the ping.
                    self.socket
                        .flush()
                        .await
                        .map_err(|error| self.transport_error("pong", error))?;
                }
                tungstenite::Message::Pong(payload) => {
                    return Ok(WebSocketEvent::Pong(payload.to_vec()));
                }
                tungstenite::Message::Close(frame) => {
                    let info = CloseInfo {
                        code: frame.as_ref().map(|frame| u16::from(frame.code)),
                        reason: frame
                            .as_ref()
                            .map(|frame| redact(&frame.reason, &self.api_key))
                            .unwrap_or_default(),
                    };
                    match self.socket.flush().await {
                        Ok(()) | Err(tungstenite::Error::ConnectionClosed) => {}
                        Err(error) => return Err(self.transport_error("close reply", error)),
                    }
                    self.closed = true;
                    return Ok(WebSocketEvent::Closed(info));
                }
                tungstenite::Message::Binary(_) | tungstenite::Message::Frame(_) => {
                    return Err(self.protocol_error("expected a JSON text frame", "binary frame"));
                }
            }
        }
    }

    fn timeout_error(&self, operation: &'static str) -> WebSocketError {
        WebSocketError::Timeout {
            operation,
            endpoint: self.endpoint.clone(),
        }
    }

    pub(super) fn protocol_error(&self, reason: &str, body: &str) -> WebSocketError {
        WebSocketError::Protocol {
            endpoint: self.endpoint.clone(),
            reason: redact(reason, &self.api_key),
            body: redact(body, &self.api_key),
        }
    }

    fn transport_error(
        &self,
        operation: &'static str,
        error: tungstenite::Error,
    ) -> WebSocketError {
        transport_error(&self.endpoint, operation, error, &self.api_key)
    }
}
