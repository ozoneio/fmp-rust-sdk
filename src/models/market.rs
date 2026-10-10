//! Analyst estimates, funds, economics, and market performance.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Analyst projections from `analyst-estimates`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalystEstimate {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    pub revenue_low: Option<f64>,
    pub revenue_high: Option<f64>,
    pub revenue_avg: Option<f64>,
    pub ebitda_low: Option<f64>,
    pub ebitda_high: Option<f64>,
    pub ebitda_avg: Option<f64>,
    pub ebit_low: Option<f64>,
    pub ebit_high: Option<f64>,
    pub ebit_avg: Option<f64>,
    pub net_income_low: Option<f64>,
    pub net_income_high: Option<f64>,
    pub net_income_avg: Option<f64>,
    pub sga_expense_low: Option<f64>,
    pub sga_expense_high: Option<f64>,
    pub sga_expense_avg: Option<f64>,
    pub eps_avg: Option<f64>,
    pub eps_high: Option<f64>,
    pub eps_low: Option<f64>,
    pub num_analysts_revenue: Option<u32>,
    pub num_analysts_eps: Option<u32>,
}

/// Consensus price targets from `price-target-consensus`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceTargetConsensus {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub target_high: Option<f64>,
    pub target_low: Option<f64>,
    pub target_consensus: Option<f64>,
    pub target_median: Option<f64>,
}

/// Fundamental rating scores from `ratings-snapshot`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingSnapshot {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub rating: String,
    pub overall_score: Option<u8>,
    pub discounted_cash_flow_score: Option<u8>,
    pub return_on_equity_score: Option<u8>,
    pub return_on_assets_score: Option<u8>,
    pub debt_to_equity_score: Option<u8>,
    pub price_to_earnings_score: Option<u8>,
    pub price_to_book_score: Option<u8>,
}

/// A discounted cash flow valuation from `discounted-cash-flow`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DiscountedCashFlow {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    pub dcf: f64,
    #[serde(rename = "Stock Price")]
    pub stock_price: Option<f64>,
}

/// An underlying holding from `etf/holdings`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtfHolding {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    /// Underlying symbol; cash and other holdings can lack a ticker.
    pub asset: String,
    pub name: String,
    pub isin: Option<String>,
    pub security_cusip: Option<String>,
    pub shares_number: Option<f64>,
    pub weight_percentage: f64,
    pub market_value: Option<f64>,
    #[serde(default, with = "super::optional_timestamp")]
    pub updated_at: Option<NaiveDateTime>,
}

/// A daily US Treasury yield observation from `treasury-rates`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TreasuryRate {
    pub date: NaiveDate,
    pub month1: Option<f64>,
    pub month2: Option<f64>,
    pub month3: Option<f64>,
    pub month6: Option<f64>,
    pub year1: Option<f64>,
    pub year2: Option<f64>,
    pub year3: Option<f64>,
    pub year5: Option<f64>,
    pub year7: Option<f64>,
    pub year10: Option<f64>,
    pub year20: Option<f64>,
    pub year30: Option<f64>,
}

/// A value for an indicator requested through `economic-indicators`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EconomicIndicator {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub name: String,
    pub date: NaiveDate,
    pub value: f64,
}

/// A scheduled or released data point from `economic-calendar`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EconomicEvent {
    #[serde(with = "super::timestamp")]
    pub date: NaiveDateTime,
    pub country: String,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub event: String,
    pub currency: Option<String>,
    pub previous: Option<f64>,
    pub estimate: Option<f64>,
    pub actual: Option<f64>,
    pub change: Option<f64>,
    pub impact: Option<String>,
    pub change_percentage: Option<f64>,
    pub unit: Option<String>,
}

/// Average changes from `sector-performance-snapshot` or `historical-sector-performance`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorPerformance {
    pub date: NaiveDate,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub sector: String,
    pub exchange: String,
    pub average_change: f64,
}

/// Average changes from `industry-performance-snapshot` or its historical endpoint.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndustryPerformance {
    pub date: NaiveDate,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub industry: String,
    pub exchange: String,
    pub average_change: f64,
}

/// A company in the `biggest-gainers`, `biggest-losers`, or `most-actives` lists.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketMover {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: Option<f64>,
    pub changes_percentage: Option<f64>,
    pub exchange: Option<String>,
}
