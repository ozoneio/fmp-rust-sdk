use serde::Serialize;
use url::Url;

use crate::plan::Subscription;

use super::{
    Connection, MarketStream, StreamConfig, StreamEntitlements, WebSocketError, WebSocketEvent,
};

/// FMP's current native WebSocket connector for approved market streams.
///
/// The [current FMP documentation](https://site.financialmodelingprep.com/developer/docs#websockets)
/// publishes `wss://socket.financialmodelingprep.com` and requires approved
/// declaration forms for direct streaming. Each stream must be explicitly
/// declared through [`StreamEntitlements`]; a billing plan alone grants none.
/// Authentication is confirmed using the documented nested login response.
#[derive(Debug)]
pub struct StandardWebSocketClient {
    connection: Connection,
    subscription: Subscription,
    entitlements: StreamEntitlements,
}

impl StandardWebSocketClient {
    /// Connects to the current native endpoint and waits for confirmed login.
    pub async fn connect(
        api_key: &str,
        subscription: &Subscription,
        entitlements: StreamEntitlements,
        config: StreamConfig,
    ) -> Result<Self, WebSocketError> {
        let endpoint = Url::parse("wss://socket.financialmodelingprep.com").map_err(|error| {
            WebSocketError::Configuration {
                reason: format!("invalid documented standard WebSocket URL: {error}"),
            }
        })?;
        Self::connect_to(api_key, subscription, entitlements, endpoint, config).await
    }

    /// Connects to a trusted explicit current endpoint; ws is loopback-only.
    ///
    /// Use this for FMP-assigned endpoints or real local integration servers.
    /// URL credentials, query parameters, and fragments are rejected.
    pub async fn connect_to(
        api_key: &str,
        subscription: &Subscription,
        entitlements: StreamEntitlements,
        endpoint: Url,
        config: StreamConfig,
    ) -> Result<Self, WebSocketError> {
        Ok(Self {
            connection: Connection::connect(api_key, endpoint, config.config).await?,
            subscription: *subscription,
            entitlements,
        })
    }

    /// Returns the configured subscription without claiming server verification.
    pub const fn subscription(&self) -> Subscription {
        self.subscription
    }

    /// Subscribes to an explicitly granted exchange, synthetic, or delayed stream.
    pub async fn subscribe(&mut self, stream: &MarketStream) -> Result<(), WebSocketError> {
        if !self.entitlements.allows(stream) {
            return Err(WebSocketError::StreamEntitlement {
                stream: stream.as_str().to_owned(),
            });
        }
        let command = StreamCommand {
            event: "subscribe",
            data: StreamData {
                stream: stream.as_str(),
            },
        };
        let json = serde_json::to_string(&command).map_err(|error| {
            self.connection
                .protocol_error(&error.to_string(), "stream subscribe serialization")
        })?;
        self.connection.send_text(json, "stream subscribe").await
    }

    /// Stops an explicitly granted stream using the documented unsubscribe event.
    pub async fn unsubscribe(&mut self, stream: &MarketStream) -> Result<(), WebSocketError> {
        if !self.entitlements.allows(stream) {
            return Err(WebSocketError::StreamEntitlement {
                stream: stream.as_str().to_owned(),
            });
        }
        let command = StreamCommand {
            event: "unsubscribe",
            data: StreamData {
                stream: stream.as_str(),
            },
        };
        let json = serde_json::to_string(&command).map_err(|error| {
            self.connection
                .protocol_error(&error.to_string(), "stream unsubscribe serialization")
        })?;
        self.connection.send_text(json, "stream unsubscribe").await
    }

    /// Receives a strictly validated snapshot, heartbeat, control, pong, or close.
    pub async fn next_event(&mut self) -> Result<WebSocketEvent, WebSocketError> {
        self.connection.next_event().await
    }

    /// Completes a close handshake within the configured handshake timeout.
    pub async fn close(self) -> Result<(), WebSocketError> {
        self.connection.close().await
    }
}

impl crate::Client {
    /// Opens an authenticated current market-stream connection for this client.
    ///
    /// Explicit stream grants are independent of the client's configured plan.
    pub async fn connect_streams(
        &self,
        entitlements: StreamEntitlements,
        config: StreamConfig,
    ) -> Result<StandardWebSocketClient, WebSocketError> {
        StandardWebSocketClient::connect(self.api_key(), self.subscription(), entitlements, config)
            .await
    }
}

#[derive(Serialize)]
struct StreamCommand<'a> {
    event: &'static str,
    data: StreamData<'a>,
}

#[derive(Serialize)]
struct StreamData<'a> {
    stream: &'a str,
}
