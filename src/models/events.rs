//! Company events, earnings calls, and news.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Earnings announced or scheduled by `earnings` and `earnings-calendar`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Earnings {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    pub eps_actual: Option<f64>,
    pub eps_estimated: Option<f64>,
    pub revenue_actual: Option<f64>,
    pub revenue_estimated: Option<f64>,
    #[serde(default, deserialize_with = "super::optional_date")]
    pub last_updated: Option<NaiveDate>,
}

/// A dividend from `dividends` or `dividends-calendar`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dividend {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    /// Ex-dividend date.
    pub date: NaiveDate,
    #[serde(default, deserialize_with = "super::optional_date")]
    pub record_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "super::optional_date")]
    pub payment_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "super::optional_date")]
    pub declaration_date: Option<NaiveDate>,
    pub adj_dividend: Option<f64>,
    pub dividend: f64,
    #[serde(rename = "yield")]
    pub dividend_yield: Option<f64>,
    pub frequency: Option<String>,
}

/// A split event from `splits` or `splits-calendar`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockSplit {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    pub numerator: f64,
    pub denominator: f64,
    pub split_type: Option<String>,
}

/// A complete call transcript from `earning-call-transcript`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EarningsTranscript {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub quarter: u8,
    pub year: u16,
    #[serde(with = "super::timestamp")]
    pub date: NaiveDateTime,
    pub content: String,
}

/// An article from the Stable news endpoints. General news can have no symbol.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsArticle {
    pub symbol: Option<String>,
    #[serde(with = "super::timestamp")]
    pub published_date: NaiveDateTime,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub title: String,
    pub publisher: Option<String>,
    pub image: Option<String>,
    pub site: Option<String>,
    pub text: Option<String>,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub url: String,
}
