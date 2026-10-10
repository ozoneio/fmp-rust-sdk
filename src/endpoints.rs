//! Typed convenience methods and explicit Stable API query options.
//!
//! Endpoint access remains authoritative at FMP: a configured subscription never
//! suppresses a request. Use `Client::get` with your own models for other Stable
//! endpoints documented by FMP.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{Client, Error, Result, models::*};

/// The reporting periods accepted by the statement endpoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Period {
    #[serde(rename = "annual")]
    Annual,
    #[serde(rename = "quarter")]
    Quarter,
    #[serde(rename = "Q1")]
    Q1,
    #[serde(rename = "Q2")]
    Q2,
    #[serde(rename = "Q3")]
    Q3,
    #[serde(rename = "Q4")]
    Q4,
}

/// The analyst estimate endpoint accepts only annual or quarterly periods.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum EstimatePeriod {
    #[serde(rename = "annual")]
    Annual,
    #[serde(rename = "quarter")]
    Quarter,
}

/// Documented intraday chart intervals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntradayInterval {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    FourHours,
}

impl IntradayInterval {
    fn path(self) -> &'static str {
        match self {
            Self::OneMinute => "historical-chart/1min",
            Self::FiveMinutes => "historical-chart/5min",
            Self::FifteenMinutes => "historical-chart/15min",
            Self::ThirtyMinutes => "historical-chart/30min",
            Self::OneHour => "historical-chart/1hour",
            Self::FourHours => "historical-chart/4hour",
        }
    }
}

/// Optional inclusive date bounds. `None` leaves the bound to FMP.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct DateRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<NaiveDate>,
}

/// An explicit page and record limit; page numbering starts at zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
}

/// Optional search filters for `search-symbol` and `search-name`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SearchOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<&'a str>,
}

/// An explicit period and number of statements, ratios, or metrics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct StatementOptions {
    pub period: Period,
    pub limit: u32,
}

/// Optional date bounds and explicit pagination for news queries.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct NewsOptions {
    #[serde(flatten)]
    pub dates: DateRange,
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Serialize)]
struct EmptyQuery {}

#[derive(Serialize)]
struct SymbolQuery<'a> {
    symbol: &'a str,
}

#[derive(Serialize)]
struct SymbolsQuery<'a> {
    symbols: &'a str,
}

#[derive(Serialize)]
struct SearchQuery<'a, 'b> {
    query: &'a str,
    #[serde(flatten)]
    options: &'b SearchOptions<'a>,
}

#[derive(Serialize)]
struct StatementQuery<'a> {
    symbol: &'a str,
    #[serde(flatten)]
    options: StatementOptions,
}

#[derive(Serialize)]
struct HistoricalQuery<'a> {
    symbol: &'a str,
    #[serde(flatten)]
    dates: DateRange,
}

#[derive(Serialize)]
struct HistoricalMarketCapQuery<'a> {
    symbol: &'a str,
    #[serde(flatten)]
    dates: DateRange,
    limit: u32,
}

#[derive(Serialize)]
struct NewsQuery<'a> {
    symbols: &'a str,
    #[serde(flatten)]
    options: NewsOptions,
}

#[derive(Serialize)]
struct AnalystQuery<'a> {
    symbol: &'a str,
    period: EstimatePeriod,
    #[serde(flatten)]
    pagination: Pagination,
}

#[derive(Serialize)]
struct TranscriptQuery<'a> {
    symbol: &'a str,
    year: u16,
    quarter: u8,
}

#[derive(Serialize)]
struct SectorSnapshotQuery<'a> {
    date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    exchange: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sector: Option<&'a str>,
}

#[derive(Serialize)]
struct IndustrySnapshotQuery<'a> {
    date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    exchange: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    industry: Option<&'a str>,
}

/// Validate a required query value before sending it to FMP.
fn required_value(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::Configuration {
            reason: format!("request parameter '{name}' must not be empty"),
        });
    }
    Ok(())
}

fn positive_limit(limit: u32) -> Result<()> {
    if limit == 0 {
        return Err(Error::Configuration {
            reason: "request parameter 'limit' must be greater than zero".to_owned(),
        });
    }
    Ok(())
}

fn valid_dates(dates: DateRange) -> Result<()> {
    if let (Some(from), Some(to)) = (dates.from, dates.to)
        && from > to
    {
        return Err(Error::Configuration {
            reason: format!("request date range is reversed: from={from}, to={to}"),
        });
    }
    Ok(())
}

fn joined_symbols(symbols: &[&str]) -> Result<String> {
    if symbols.is_empty() {
        return Err(Error::Configuration {
            reason: "request parameter 'symbols' must contain at least one symbol".to_owned(),
        });
    }
    for symbol in symbols {
        required_value("symbols", symbol)?;
        if symbol.contains(',') {
            return Err(Error::Configuration {
                reason: format!("each batch symbol must be a single ticker, received '{symbol}'"),
            });
        }
    }
    Ok(symbols.join(","))
}

macro_rules! symbol_endpoint {
    ($name:ident, $path:literal, $response:ty, $doc:literal) => {
        impl Client {
            #[doc = $doc]
            pub async fn $name(&self, symbol: &str) -> Result<Vec<$response>> {
                required_value("symbol", symbol)?;
                self.get($path, &SymbolQuery { symbol }).await
            }
        }
    };
}

macro_rules! batch_endpoint {
    ($name:ident, $path:literal, $response:ty, $doc:literal) => {
        impl Client {
            #[doc = $doc]
            pub async fn $name(&self, symbols: &[&str]) -> Result<Vec<$response>> {
                let symbols = joined_symbols(symbols)?;
                self.get($path, &SymbolsQuery { symbols: &symbols }).await
            }
        }
    };
}

macro_rules! directory_endpoint {
    ($name:ident, $path:literal, $response:ty, $doc:literal) => {
        impl Client {
            #[doc = $doc]
            pub async fn $name(&self) -> Result<Vec<$response>> {
                self.get($path, &EmptyQuery {}).await
            }
        }
    };
}

macro_rules! statement_endpoint {
    ($name:ident, $path:literal, $response:ty, $doc:literal) => {
        impl Client {
            #[doc = $doc]
            pub async fn $name(
                &self,
                symbol: &str,
                options: StatementOptions,
            ) -> Result<Vec<$response>> {
                required_value("symbol", symbol)?;
                positive_limit(options.limit)?;
                self.get($path, &StatementQuery { symbol, options }).await
            }
        }
    };
}

mod methods;
