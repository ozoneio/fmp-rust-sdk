use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite};
use url::Url;

use super::streams::validate_stream_quote;
use super::{
    AuthenticationStatus, ControlEvent, Heartbeat, MarketUpdate, Socket, StreamQuote,
    WebSocketCluster, WebSocketConfig, WebSocketError, WebSocketEvent,
};
use crate::{credentials::redact, plan::Subscription};

#[derive(Serialize)]
pub(super) struct LoginCommand<'a> {
    pub(super) event: &'static str,
    pub(super) data: LoginData<'a>,
}

#[derive(Serialize)]
pub(super) struct LoginData<'a> {
    #[serde(rename = "apiKey")]
    pub(super) api_key: &'a str,
}

#[derive(Serialize)]
pub(super) struct TickerCommand {
    pub(super) event: &'static str,
    pub(super) data: TickerData,
}

#[derive(Serialize)]
pub(super) struct TickerData {
    pub(super) ticker: Vec<String>,
}

#[derive(Deserialize)]
struct EventHeader {
    event: Option<String>,
}

#[derive(Deserialize)]
struct TopStatusFields {
    status: Option<u16>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct QuoteHeader {
    symbol: Option<String>,
}

#[derive(Deserialize)]
struct ControlStatusFields {
    data: Option<NestedStatusFields>,
}

#[derive(Deserialize)]
struct NestedStatusFields {
    status: Option<u16>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct StandardLoginAcknowledgment {
    event: String,
    data: AuthenticationStatus,
}

pub(super) fn validate_connection(
    api_key: &str,
    subscription: &Subscription,
    cluster: WebSocketCluster,
    endpoint: &Url,
) -> Result<(), WebSocketError> {
    let grants = subscription.websocket_entitlements();
    let allowed = match cluster {
        WebSocketCluster::Stocks => grants.stocks,
        WebSocketCluster::Crypto => grants.crypto,
        WebSocketCluster::Forex => grants.forex,
    };
    if !allowed {
        return Err(WebSocketError::Entitlement { cluster });
    }
    validate_endpoint(api_key, endpoint)
}

pub(super) fn validate_endpoint(api_key: &str, endpoint: &Url) -> Result<(), WebSocketError> {
    if api_key.is_empty()
        || api_key
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        return Err(WebSocketError::Configuration {
            reason: "api_key must be nonempty and contain no whitespace or control characters"
                .to_owned(),
        });
    }
    if !matches!(endpoint.scheme(), "ws" | "wss")
        || endpoint.host_str().is_none()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
    {
        return Err(WebSocketError::Configuration {
            reason: "endpoint must be ws/wss with a host and no credentials, query, or fragment"
                .to_owned(),
        });
    }
    if endpoint.scheme() == "ws" && !is_loopback(endpoint) {
        return Err(WebSocketError::Configuration {
            reason: "unencrypted ws endpoints are permitted only for loopback integration servers; use wss for remote servers".to_owned(),
        });
    }
    Ok(())
}

fn is_loopback(endpoint: &Url) -> bool {
    match endpoint.host() {
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(address)) => IpAddr::V4(address).is_loopback(),
        Some(url::Host::Ipv6(address)) => IpAddr::V6(address).is_loopback(),
        None => false,
    }
}

pub(super) fn normalize_symbols(symbols: &[&str]) -> Result<Vec<String>, WebSocketError> {
    if symbols.is_empty()
        || symbols.iter().any(|symbol| {
            symbol.is_empty()
                || !symbol.is_ascii()
                || symbol
                    .chars()
                    .any(|character| character.is_whitespace() || character.is_control())
        })
    {
        return Err(WebSocketError::Configuration {
            reason: "tickers must be a nonempty list of nonempty ASCII symbols without whitespace or control characters".to_owned(),
        });
    }
    Ok(symbols
        .iter()
        .map(|symbol| symbol.to_ascii_lowercase())
        .collect())
}

pub(super) fn validate_update(update: &MarketUpdate) -> Result<(), &'static str> {
    let (symbol, valid_values) = match update {
        MarketUpdate::Trade(trade) | MarketUpdate::Break(trade) => (
            &trade.symbol,
            [trade.price, trade.size]
                .iter()
                .all(|value| value.is_finite() && *value >= 0.0),
        ),
        MarketUpdate::Quote(quote) => (
            &quote.symbol,
            [
                quote.ask_price,
                quote.ask_size,
                quote.bid_price,
                quote.bid_size,
            ]
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0),
        ),
    };
    if symbol.is_empty()
        || symbol
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        return Err(
            "market symbol must be nonempty and contain no whitespace or control characters",
        );
    }
    if !valid_values {
        return Err("market prices and sizes must be finite and nonnegative");
    }
    Ok(())
}

pub(super) async fn connect_with_retries(
    endpoint: &str,
    config: &WebSocketConfig,
    api_key: &str,
) -> Result<Socket, WebSocketError> {
    for attempt in 1..=config.connect_attempts {
        let (error, retryable) =
            match timeout(config.handshake_timeout, connect_async(endpoint)).await {
                Ok(Ok((socket, _))) => return Ok(socket),
                Ok(Err(error)) => {
                    let retryable = match &error {
                        tungstenite::Error::Io(_) | tungstenite::Error::Tls(_) => true,
                        tungstenite::Error::Http(response) => {
                            response.status().is_server_error() || response.status().as_u16() == 429
                        }
                        _ => false,
                    };
                    (
                        transport_error(endpoint, "connect", error, api_key),
                        retryable,
                    )
                }
                Err(_) => (
                    WebSocketError::Timeout {
                        operation: "connect",
                        endpoint: redact(endpoint, api_key),
                    },
                    true,
                ),
            };
        if !retryable || attempt == config.connect_attempts {
            return Err(error);
        }
        tracing::warn!(
            endpoint = %redact(endpoint, api_key),
            attempt,
            max_attempts = config.connect_attempts,
            error = %error,
            "FMP WebSocket connection failed; retrying"
        );
        tokio::time::sleep(config.retry_delay).await;
    }
    Err(WebSocketError::Configuration {
        reason: "connect_attempts must be positive".to_owned(),
    })
}

pub(super) fn transport_error(
    endpoint: &str,
    operation: &'static str,
    error: tungstenite::Error,
    api_key: &str,
) -> WebSocketError {
    let reason = match &error {
        tungstenite::Error::Http(response) => format!(
            "HTTP {}; body: {}",
            response.status(),
            response
                .body()
                .as_ref()
                .map(|body| String::from_utf8_lossy(body).into_owned())
                .unwrap_or_default()
        ),
        _ => error.to_string(),
    };
    WebSocketError::Transport {
        operation,
        endpoint: redact(endpoint, api_key),
        reason: redact(&reason, api_key),
    }
}

pub(super) fn parse_event(
    endpoint: &str,
    api_key: &str,
    json: &str,
) -> Result<WebSocketEvent, WebSocketError> {
    let header: EventHeader = serde_json::from_str(json)
        .map_err(|error| protocol_error(endpoint, api_key, &error.to_string(), json))?;
    if let Some(event) = header.event {
        if event.is_empty()
            || event
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(protocol_error(
                endpoint,
                api_key,
                "control event must be nonempty and contain no whitespace or control characters",
                json,
            ));
        }
        let (status, message) = control_status(endpoint, api_key, json)?;
        let control = ControlEvent {
            event: redact(&event, api_key),
            status,
            message: message.map(|message| redact(&message, api_key)),
            json: redact(json, api_key),
        };
        if control.event == "error" || control.status.is_some_and(|status| status >= 400) {
            return Err(WebSocketError::Server {
                event: control.event,
                status: control.status,
                endpoint: endpoint.to_owned(),
                body: control.json,
            });
        }
        if control.event == "heartbeat" {
            let heartbeat: Heartbeat = serde_json::from_str(json)
                .map_err(|error| protocol_error(endpoint, api_key, &error.to_string(), json))?;
            return Ok(WebSocketEvent::Heartbeat(heartbeat));
        }
        return Ok(WebSocketEvent::Control(control));
    }
    let quote_header: QuoteHeader = serde_json::from_str(json)
        .map_err(|error| protocol_error(endpoint, api_key, &error.to_string(), json))?;
    if quote_header.symbol.is_some() {
        let quote: StreamQuote = serde_json::from_str(json)
            .map_err(|error| protocol_error(endpoint, api_key, &error.to_string(), json))?;
        validate_stream_quote(&quote)
            .map_err(|reason| protocol_error(endpoint, api_key, reason, json))?;
        return Ok(WebSocketEvent::StreamQuote(Box::new(quote)));
    }
    let update: MarketUpdate = serde_json::from_str(json)
        .map_err(|error| protocol_error(endpoint, api_key, &error.to_string(), json))?;
    validate_update(&update).map_err(|reason| protocol_error(endpoint, api_key, reason, json))?;
    Ok(WebSocketEvent::Market(update))
}

fn control_status(
    endpoint: &str,
    api_key: &str,
    json: &str,
) -> Result<(Option<u16>, Option<String>), WebSocketError> {
    let fields: ControlStatusFields = serde_json::from_str(json)
        .map_err(|error| protocol_error(endpoint, api_key, &error.to_string(), json))?;
    if let Some(data) = fields.data
        && let Some(status) = data.status
    {
        let message = data.message.ok_or_else(|| {
            protocol_error(
                endpoint,
                api_key,
                "nested control status requires data.message",
                json,
            )
        })?;
        if message.is_empty() {
            return Err(protocol_error(
                endpoint,
                api_key,
                "nested control data.message must not be empty",
                json,
            ));
        }
        return Ok((Some(status), Some(message)));
    }
    let top: TopStatusFields = serde_json::from_str(json)
        .map_err(|error| protocol_error(endpoint, api_key, &error.to_string(), json))?;
    Ok((top.status, top.message))
}

pub(super) fn confirm_standard_login(control: &ControlEvent) -> Result<bool, WebSocketError> {
    if control.event != "login" {
        return Ok(false);
    }
    let acknowledgment: StandardLoginAcknowledgment =
        serde_json::from_str(&control.json).map_err(|error| WebSocketError::Configuration {
            reason: format!(
                "standard login acknowledgment requires event and data.status/data.message: {error}"
            ),
        })?;
    if acknowledgment.data.message.is_empty() {
        return Err(WebSocketError::Configuration {
            reason: "standard login data.message must not be empty".to_owned(),
        });
    }
    Ok(acknowledgment.event == "login" && acknowledgment.data.status == 200)
}

pub(super) fn protocol_error(
    endpoint: &str,
    api_key: &str,
    reason: &str,
    body: &str,
) -> WebSocketError {
    WebSocketError::Protocol {
        endpoint: redact(endpoint, api_key),
        reason: redact(reason, api_key),
        body: redact(body, api_key),
    }
}
