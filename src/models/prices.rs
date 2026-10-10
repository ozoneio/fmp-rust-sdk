//! End-of-day and intraday price series.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// A daily OHLCV bar from `historical-price-eod/full`; prices are split-adjusted.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalPrice {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub vwap: Option<f64>,
}

/// A daily close from `historical-price-eod/light`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct HistoricalPriceLight {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    pub price: f64,
    pub volume: Option<f64>,
}

/// An intraday OHLCV bar from `historical-chart/{interval}`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct IntradayPrice {
    /// The provider's timestamp has no timezone offset.
    #[serde(with = "super::timestamp")]
    pub date: NaiveDateTime,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
