use std::{fmt, sync::Arc, time::Duration};

use super::{Heartbeat, StreamQuote};
use serde::Deserialize;
use thiserror::Error;

/// The FMP market cluster to connect to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketCluster {
    /// The stock trade and quote feed.
    Stocks,
    /// The cryptocurrency trade and quote feed.
    Crypto,
    /// The forex trade and quote feed.
    Forex,
}

impl WebSocketCluster {
    /// Returns the endpoint published in FMP's public WebSocket documentation.
    pub const fn endpoint(self) -> &'static str {
        match self {
            Self::Stocks => "wss://websockets.financialmodelingprep.com",
            Self::Crypto => "wss://crypto.financialmodelingprep.com",
            Self::Forex => "wss://forex.financialmodelingprep.com",
        }
    }
}

/// A validated JSON control envelope with its original, credential-redacted JSON.
///
/// FMP's public documentation does not prescribe every control payload.
/// `status` and `message` are populated when present at the top level; additional
/// fields can be decoded from `json` into an account-specific strict model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlEvent {
    /// The required nonempty event name.
    pub event: String,
    /// The server status, from a top-level control or nested login data.
    pub status: Option<u16>,
    /// The server message, from a top-level control or nested login data.
    pub message: Option<String>,
    /// The complete control envelope, with occurrences of the API key redacted.
    pub json: String,
}

/// Validates an account's login acknowledgment.
///
/// Return `Ok(true)` only for a positively validated authentication success,
/// `Ok(false)` for other controls, and an error for a malformed acknowledgment.
/// An unknown event or absence of an error is not authentication success.
pub type LoginConfirmation =
    Arc<dyn Fn(&ControlEvent) -> Result<bool, WebSocketError> + Send + Sync>;

/// Explicit timeouts and connection retry settings.
#[derive(Clone)]
pub struct WebSocketConfig {
    pub(super) handshake_timeout: Duration,
    pub(super) receive_timeout: Duration,
    pub(super) connect_attempts: u32,
    pub(super) retry_delay: Duration,
    pub(super) confirm_login: LoginConfirmation,
}

impl fmt::Debug for WebSocketConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebSocketConfig")
            .field("handshake_timeout", &self.handshake_timeout)
            .field("receive_timeout", &self.receive_timeout)
            .field("connect_attempts", &self.connect_attempts)
            .field("retry_delay", &self.retry_delay)
            .finish_non_exhaustive()
    }
}

impl WebSocketConfig {
    /// Creates a configuration; timeouts and attempt count must be positive.
    ///
    /// Only transient HTTP upgrade/transport failures are retried, before login.
    /// Authentication, protocol, and active connection failures are returned.
    pub fn new(
        handshake_timeout: Duration,
        receive_timeout: Duration,
        connect_attempts: u32,
        retry_delay: Duration,
        confirm_login: impl Fn(&ControlEvent) -> Result<bool, WebSocketError> + Send + Sync + 'static,
    ) -> Result<Self, WebSocketError> {
        if handshake_timeout.is_zero() || receive_timeout.is_zero() || connect_attempts == 0 {
            return Err(WebSocketError::Configuration {
                reason: "handshake_timeout, receive_timeout, and connect_attempts must be positive"
                    .to_owned(),
            });
        }
        Ok(Self {
            handshake_timeout,
            receive_timeout,
            connect_attempts,
            retry_delay,
            confirm_login: Arc::new(confirm_login),
        })
    }
}

/// Required fields of a trade or trade-break update.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TradeUpdate {
    /// The asset's symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// The provider timestamp, preserved without assuming a time unit.
    #[serde(rename = "t")]
    pub timestamp: u64,
    /// The execution price.
    #[serde(rename = "lp")]
    pub price: f64,
    /// The traded quantity; crypto quantities can be fractional.
    #[serde(rename = "ls")]
    pub size: f64,
}

/// Required fields of a top-of-book quote update.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct QuoteUpdate {
    /// The asset's symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// The provider timestamp, preserved without assuming a time unit.
    #[serde(rename = "t")]
    pub timestamp: u64,
    /// The lowest ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// The quantity at the ask price.
    #[serde(rename = "as")]
    pub ask_size: f64,
    /// The highest bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// The quantity at the bid price.
    #[serde(rename = "bs")]
    pub bid_size: f64,
}

/// A strictly typed update using FMP's documented T/Q/B discriminator.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum MarketUpdate {
    /// A last-trade execution.
    #[serde(rename = "T")]
    Trade(TradeUpdate),
    /// A top-of-book quote.
    #[serde(rename = "Q")]
    Quote(QuoteUpdate),
    /// A trade-break update.
    #[serde(rename = "B")]
    Break(TradeUpdate),
}

/// The remote peer's completed close handshake.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseInfo {
    /// The close code, if the peer supplied a close frame payload.
    pub code: Option<u16>,
    /// The peer's reason, with credentials redacted.
    pub reason: String,
}

/// An incoming application, pong, or close event.
#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketEvent {
    /// A validated trade, quote, or trade-break update.
    Market(MarketUpdate),
    /// A current market stream's validated quote snapshot.
    StreamQuote(Box<StreamQuote>),
    /// The current server's required timestamped heartbeat.
    Heartbeat(Heartbeat),
    /// A control message; server error controls instead return an error.
    Control(ControlEvent),
    /// The peer's pong payload.
    Pong(Vec<u8>),
    /// A cleanly closed connection.
    Closed(CloseInfo),
}

/// A specific connection, configuration, server, or protocol failure.
#[derive(Debug, Error)]
pub enum WebSocketError {
    /// Invalid local configuration or subscription symbols.
    #[error("invalid WebSocket configuration: {reason}")]
    Configuration {
        /// The actionable validation failure.
        reason: String,
    },
    /// The account has no configured grant for this cluster.
    #[error(
        "no explicit WebSocket entitlement configured for {cluster:?}; confirm the cluster grant with FMP"
    )]
    Entitlement {
        /// The requested cluster.
        cluster: WebSocketCluster,
    },
    /// The requested current market stream has no configured grant.
    #[error(
        "no explicit WebSocket entitlement configured for stream {stream}; confirm the stream grant with FMP"
    )]
    StreamEntitlement {
        /// The requested stream identifier.
        stream: String,
    },
    /// A bounded operation exceeded its configured timeout.
    #[error("WebSocket {operation} timed out for {endpoint}")]
    Timeout {
        /// The operation that exceeded its deadline.
        operation: &'static str,
        /// The requested endpoint.
        endpoint: String,
    },
    /// An HTTP upgrade or active transport operation failed.
    #[error("WebSocket {operation} failed for {endpoint}: {reason}")]
    Transport {
        /// The failed operation.
        operation: &'static str,
        /// The requested endpoint.
        endpoint: String,
        /// The underlying error, including HTTP status/body when available.
        reason: String,
    },
    /// A server control explicitly rejected a request or authentication.
    #[error("FMP WebSocket rejected {event} at {endpoint} (status {status:?}): {body}")]
    Server {
        /// The failed control event.
        event: String,
        /// The provider's top-level status code, if supplied.
        status: Option<u16>,
        /// The requested endpoint.
        endpoint: String,
        /// The complete redacted server control payload.
        body: String,
    },
    /// The remote payload failed strict runtime validation.
    #[error("invalid WebSocket response at {endpoint}: {reason}; body: {body}")]
    Protocol {
        /// The requested endpoint.
        endpoint: String,
        /// The exact validation failure.
        reason: String,
        /// The credential-redacted payload.
        body: String,
    },
    /// A command was attempted after a completed remote close.
    #[error(
        "WebSocket connection is closed at {endpoint}; reconnect explicitly before issuing commands"
    )]
    Closed {
        /// The closed connection's endpoint.
        endpoint: String,
    },
}
