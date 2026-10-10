use std::{collections::BTreeSet, fmt, str::FromStr, time::Duration};

use serde::Deserialize;

use super::{WebSocketConfig, WebSocketError};

const STREAM_NAMES: [&str; 14] = [
    "nasdaq-basic-w-nls-plus",
    "iex-tops",
    "tsx-level-1",
    "tsxv-level-1",
    "lse-level-1",
    "cboe-index-main",
    "fmp-crypto-stream",
    "fmp-index-stream",
    "fmp-commodity-stream",
    "fmp-currency-stream",
    "fmp-us-equities-stream",
    "fmp-us-otc-stream",
    "fmp-uk-equities-stream",
    "fmp-ca-equities-stream",
];

/// A documented standard WebSocket stream, including its delayed variant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MarketStream(String);

impl MarketStream {
    /// Returns the exact stream identifier sent to FMP.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for MarketStream {
    type Err = WebSocketError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let name = input.strip_suffix("-delayed").unwrap_or(input);
        if !STREAM_NAMES.contains(&name) {
            return Err(WebSocketError::Configuration {
                reason: format!(
                    "unknown FMP WebSocket stream {input:?}; use a stream listed in FMP's standard WebSocket documentation, optionally ending in -delayed"
                ),
            });
        }
        Ok(Self(input.to_owned()))
    }
}

impl fmt::Display for MarketStream {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Explicit stream grants confirmed in the account dashboard or with FMP.
///
/// A billing plan does not imply access to an exchange or delayed feed. The
/// server remains authoritative about each configured grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamEntitlements {
    streams: BTreeSet<MarketStream>,
}

impl StreamEntitlements {
    /// Creates an immutable allowlist containing at least one granted stream.
    pub fn new(streams: Vec<MarketStream>) -> Result<Self, WebSocketError> {
        if streams.is_empty() {
            return Err(WebSocketError::Configuration {
                reason: "configure at least one explicitly granted FMP WebSocket stream".to_owned(),
            });
        }
        Ok(Self {
            streams: streams.into_iter().collect(),
        })
    }

    /// Returns whether this exact real-time or delayed stream has a grant.
    pub fn allows(&self, stream: &MarketStream) -> bool {
        self.streams.contains(stream)
    }
}

/// Explicit timeouts and retry settings for the standard stream protocol.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub(super) config: WebSocketConfig,
}

impl StreamConfig {
    /// Creates a configuration using FMP's documented nested login response.
    ///
    /// Timeouts and attempt count must be positive. Transient connection
    /// failures are retried before login; active connections require an explicit
    /// reconnect after failure.
    pub fn new(
        handshake_timeout: Duration,
        receive_timeout: Duration,
        connect_attempts: u32,
        retry_delay: Duration,
    ) -> Result<Self, WebSocketError> {
        Ok(Self {
            config: WebSocketConfig::new(
                handshake_timeout,
                receive_timeout,
                connect_attempts,
                retry_delay,
                super::protocol::confirm_standard_login,
            )?,
        })
    }
}

/// Required fields in the standard protocol's nested login response.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AuthenticationStatus {
    /// The provider's authentication status code; success is 200.
    pub status: u16,
    /// The provider's authentication message.
    pub message: String,
}

/// A standard stream heartbeat with the provider's required timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Heartbeat {
    /// The provider timestamp, preserved without assuming a time unit.
    pub timestamp: u64,
}

/// A standard stream quote with required symbol, price, and timestamp.
///
/// Additional quote metadata depends on the asset and stream. These fields
/// accept null or absence and are validated when supplied. Unrelated provider
/// fields are ignored.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamQuote {
    /// The asset's required nonempty symbol.
    pub symbol: String,
    /// The asset's name, when supplied.
    pub name: Option<String>,
    /// The current price.
    pub price: f64,
    /// The price change as a percentage.
    pub changes_percentage: Option<f64>,
    /// The signed price change.
    pub change: Option<f64>,
    /// The current day's lowest price.
    pub day_low: Option<f64>,
    /// The current day's highest price.
    pub day_high: Option<f64>,
    /// The past year's lowest price.
    pub year_low: Option<f64>,
    /// The past year's highest price.
    pub year_high: Option<f64>,
    /// The market capitalization.
    pub market_cap: Option<f64>,
    /// The 50-day average price.
    pub price_avg50: Option<f64>,
    /// The 200-day average price.
    pub price_avg200: Option<f64>,
    /// The exchange identifier, when supplied.
    pub exchange: Option<String>,
    /// The traded volume; fractional quantities are preserved.
    pub volume: Option<f64>,
    /// The average traded volume.
    pub avg_volume: Option<f64>,
    /// The opening price.
    pub open: Option<f64>,
    /// The previous closing price.
    pub previous_close: Option<f64>,
    /// Earnings per share, when available.
    pub eps: Option<f64>,
    /// The price-to-earnings ratio, when available.
    pub pe: Option<f64>,
    /// The provider's earnings announcement timestamp, when available.
    pub earnings_announcement: Option<String>,
    /// Outstanding shares, when available.
    pub shares_outstanding: Option<f64>,
    /// The provider timestamp, preserved without assuming a time unit.
    pub timestamp: u64,
}

pub(super) fn validate_stream_quote(quote: &StreamQuote) -> Result<(), &'static str> {
    if quote.symbol.is_empty()
        || quote
            .symbol
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        return Err(
            "quote symbol must be nonempty and contain no whitespace or control characters",
        );
    }
    for value in [quote.name.as_ref(), quote.exchange.as_ref()]
        .into_iter()
        .flatten()
    {
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(
                "quote name and exchange must be nonempty and contain no control characters when supplied",
            );
        }
    }
    for value in [
        Some(quote.price),
        quote.day_low,
        quote.day_high,
        quote.year_low,
        quote.year_high,
        quote.price_avg50,
        quote.price_avg200,
        quote.open,
        quote.previous_close,
    ]
    .into_iter()
    .flatten()
    {
        if !value.is_finite() {
            return Err("quote prices must be finite; negative commodity prices are permitted");
        }
    }
    for value in [
        quote.market_cap,
        quote.volume,
        quote.avg_volume,
        quote.shares_outstanding,
    ]
    .into_iter()
    .flatten()
    {
        if !value.is_finite() || value < 0.0 {
            return Err(
                "quote market capitalization, volumes, and outstanding shares must be finite and nonnegative",
            );
        }
    }
    for value in [quote.change, quote.changes_percentage, quote.eps, quote.pe]
        .into_iter()
        .flatten()
    {
        if !value.is_finite() {
            return Err(
                "quote changes, earnings per share, and price-to-earnings ratio must be finite",
            );
        }
    }
    if quote
        .earnings_announcement
        .as_ref()
        .is_some_and(|value| value.trim().is_empty() || value.chars().any(char::is_control))
    {
        return Err(
            "quote earnings announcement must be nonempty and contain no control characters when supplied",
        );
    }
    Ok(())
}
