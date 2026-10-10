//! Current market-stream and legacy cluster WebSocket connections.
//!
//! [`StandardWebSocketClient`] supports FMP's current native WebSocket at
//! `wss://socket.financialmodelingprep.com`, with approved exchange, synthetic,
//! and delayed streams. [`StreamEntitlements`] explicitly records the approved
//! streams, while [`Subscription`] records the separately configured plan.
//!
//! The legacy [`WebSocketClient`] supports stock, crypto, and forex clusters.
//! Legacy public docs omit successful login acknowledgments, so its config
//! requires an account-specific [`LoginConfirmation`]. Both connectors wait
//! for confirmed authentication before allowing subscriptions and report
//! active connection failures without silently reconnecting.

mod connection;
mod protocol;
mod standard;
mod streams;
mod types;

pub use types::{
    CloseInfo, ControlEvent, LoginConfirmation, MarketUpdate, QuoteUpdate, TradeUpdate,
    WebSocketCluster, WebSocketConfig, WebSocketError, WebSocketEvent,
};

use crate::plan::Subscription;
use connection::{Connection, Socket};
use protocol::{TickerCommand, TickerData, normalize_symbols, validate_connection};
use url::Url;

pub use standard::StandardWebSocketClient;
pub use streams::{
    AuthenticationStatus, Heartbeat, MarketStream, StreamConfig, StreamEntitlements, StreamQuote,
};

/// An authenticated connector to FMP's legacy stock, crypto, or forex cluster.
///
/// For current market streams, use [`StandardWebSocketClient`].
#[derive(Debug)]
pub struct WebSocketClient {
    connection: Connection,
    cluster: WebSocketCluster,
}

impl WebSocketClient {
    /// Connects to the documented legacy cluster endpoint and confirms login.
    pub async fn connect(
        api_key: &str,
        subscription: &Subscription,
        cluster: WebSocketCluster,
        config: WebSocketConfig,
    ) -> Result<Self, WebSocketError> {
        let endpoint =
            Url::parse(cluster.endpoint()).map_err(|error| WebSocketError::Configuration {
                reason: format!("invalid documented cluster URL: {error}"),
            })?;
        Self::connect_to(api_key, subscription, cluster, endpoint, config).await
    }

    /// Connects to a trusted explicit legacy endpoint; ws is loopback-only.
    pub async fn connect_to(
        api_key: &str,
        subscription: &Subscription,
        cluster: WebSocketCluster,
        endpoint: Url,
        config: WebSocketConfig,
    ) -> Result<Self, WebSocketError> {
        validate_connection(api_key, subscription, cluster, &endpoint)?;
        Ok(Self {
            connection: Connection::connect(api_key, endpoint, config).await?,
            cluster,
        })
    }

    /// Returns the legacy cluster used by this connector.
    pub const fn cluster(&self) -> WebSocketCluster {
        self.cluster
    }

    /// Subscribes to validated lowercase legacy ticker symbols.
    pub async fn subscribe(&mut self, symbols: &[&str]) -> Result<(), WebSocketError> {
        let command = TickerCommand {
            event: "subscribe",
            data: TickerData {
                ticker: normalize_symbols(symbols)?,
            },
        };
        let json = serde_json::to_string(&command).map_err(|error| {
            self.connection
                .protocol_error(&error.to_string(), "subscribe serialization")
        })?;
        self.connection.send_text(json, "subscribe").await
    }

    /// Unsubscribes from validated lowercase legacy ticker symbols.
    pub async fn unsubscribe(&mut self, symbols: &[&str]) -> Result<(), WebSocketError> {
        let command = TickerCommand {
            event: "unsubscribe",
            data: TickerData {
                ticker: normalize_symbols(symbols)?,
            },
        };
        let json = serde_json::to_string(&command).map_err(|error| {
            self.connection
                .protocol_error(&error.to_string(), "unsubscribe serialization")
        })?;
        self.connection.send_text(json, "unsubscribe").await
    }

    /// Receives a typed update, control, pong, or remote close event.
    pub async fn next_event(&mut self) -> Result<WebSocketEvent, WebSocketError> {
        self.connection.next_event().await
    }

    /// Completes a close handshake within the configured timeout.
    pub async fn close(self) -> Result<(), WebSocketError> {
        self.connection.close().await
    }
}
