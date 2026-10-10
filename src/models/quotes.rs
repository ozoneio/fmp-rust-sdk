//! Current and aftermarket market quotes.

use serde::{Deserialize, Serialize};

/// A full quote from `quote` or `batch-quote`, including supported non-stock symbols.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub price: f64,
    pub name: Option<String>,
    pub change_percentage: Option<f64>,
    pub change: Option<f64>,
    pub volume: Option<f64>,
    pub day_low: Option<f64>,
    pub day_high: Option<f64>,
    pub year_high: Option<f64>,
    pub year_low: Option<f64>,
    pub market_cap: Option<f64>,
    #[serde(rename = "priceAvg50")]
    pub price_avg_50: Option<f64>,
    #[serde(rename = "priceAvg200")]
    pub price_avg_200: Option<f64>,
    pub exchange: Option<String>,
    pub open: Option<f64>,
    pub previous_close: Option<f64>,
    /// Unix seconds, unlike aftermarket timestamps which are milliseconds.
    pub timestamp: Option<u64>,
}

/// A compact quote from `quote-short` or `batch-quote-short`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ShortQuote {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub price: f64,
    pub change: Option<f64>,
    pub volume: Option<f64>,
}

/// An after-hours trade from `aftermarket-trade`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AftermarketTrade {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub price: f64,
    pub trade_size: Option<f64>,
    /// Unix milliseconds.
    pub timestamp: u64,
}

/// An after-hours bid/ask snapshot from `aftermarket-quote`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AftermarketQuote {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub bid_size: Option<f64>,
    pub bid_price: Option<f64>,
    pub ask_size: Option<f64>,
    pub ask_price: Option<f64>,
    pub volume: Option<f64>,
    /// Unix milliseconds.
    pub timestamp: u64,
}

/// Price changes for the horizons returned by `stock-price-change`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PriceChange {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    #[serde(rename = "1D")]
    pub one_day: Option<f64>,
    #[serde(rename = "5D")]
    pub five_days: Option<f64>,
    #[serde(rename = "1M")]
    pub one_month: Option<f64>,
    #[serde(rename = "3M")]
    pub three_months: Option<f64>,
    #[serde(rename = "6M")]
    pub six_months: Option<f64>,
    #[serde(rename = "ytd")]
    pub year_to_date: Option<f64>,
    #[serde(rename = "1Y")]
    pub one_year: Option<f64>,
    #[serde(rename = "3Y")]
    pub three_years: Option<f64>,
    #[serde(rename = "5Y")]
    pub five_years: Option<f64>,
    #[serde(rename = "10Y")]
    pub ten_years: Option<f64>,
    #[serde(rename = "max")]
    pub all_time: Option<f64>,
}
