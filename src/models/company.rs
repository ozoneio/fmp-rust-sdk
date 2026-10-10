//! Search, directory, and company information responses.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// A match from `search-symbol` or `search-name`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub name: String,
    pub currency: Option<String>,
    pub exchange_full_name: Option<String>,
    pub exchange: Option<String>,
}

/// A symbol and name from `stock-list`, `etf-list`, or `actively-trading-list`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DirectorySymbol {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub name: String,
}

/// Company information returned by the Stable `profile` endpoint.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub company_name: String,
    pub price: Option<f64>,
    pub market_cap: Option<f64>,
    pub beta: Option<f64>,
    pub last_dividend: Option<f64>,
    pub range: Option<String>,
    pub change: Option<f64>,
    pub change_percentage: Option<f64>,
    pub volume: Option<f64>,
    pub average_volume: Option<f64>,
    pub currency: Option<String>,
    pub cik: Option<String>,
    pub isin: Option<String>,
    pub cusip: Option<String>,
    pub exchange_full_name: Option<String>,
    pub exchange: Option<String>,
    pub industry: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub ceo: Option<String>,
    pub sector: Option<String>,
    pub country: Option<String>,
    /// FMP reports the employee count as a string.
    pub full_time_employees: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub image: Option<String>,
    #[serde(default, deserialize_with = "super::optional_date")]
    pub ipo_date: Option<NaiveDate>,
    pub default_image: Option<bool>,
    pub is_etf: Option<bool>,
    pub is_actively_trading: Option<bool>,
    pub is_adr: Option<bool>,
    pub is_fund: Option<bool>,
}

/// A comparable company from `stock-peers`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub company_name: String,
    pub price: Option<f64>,
    #[serde(rename = "mktCap")]
    pub market_cap: Option<f64>,
}

/// A daily market capitalization from `market-capitalization` or its historical endpoint.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCapitalization {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    pub market_cap: f64,
}

/// Share ownership and available float from `shares-float`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareFloat {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    #[serde(with = "super::timestamp")]
    pub date: NaiveDateTime,
    pub free_float: Option<f64>,
    pub float_shares: Option<f64>,
    pub outstanding_shares: Option<f64>,
    pub source: Option<String>,
}
