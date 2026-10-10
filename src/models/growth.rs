//! Growth of financial statement values.

use super::FinancialPeriod;
use serde::{Deserialize, Serialize};

/// Historical growth rates from `financial-growth`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialGrowth {
    #[serde(flatten)]
    pub metadata: FinancialPeriod,
    pub revenue_growth: Option<f64>,
    pub gross_profit_growth: Option<f64>,
    #[serde(rename = "ebitgrowth")]
    pub ebit_growth: Option<f64>,
    pub operating_income_growth: Option<f64>,
    pub net_income_growth: Option<f64>,
    #[serde(rename = "epsgrowth")]
    pub eps_growth: Option<f64>,
    #[serde(rename = "epsdilutedGrowth")]
    pub eps_diluted_growth: Option<f64>,
    pub weighted_average_shares_growth: Option<f64>,
    pub weighted_average_shares_diluted_growth: Option<f64>,
    pub dividends_per_share_growth: Option<f64>,
    pub operating_cash_flow_growth: Option<f64>,
    pub receivables_growth: Option<f64>,
    pub inventory_growth: Option<f64>,
    pub asset_growth: Option<f64>,
    #[serde(rename = "bookValueperShareGrowth")]
    pub book_value_per_share_growth: Option<f64>,
    pub debt_growth: Option<f64>,
    #[serde(rename = "rdexpenseGrowth")]
    pub research_and_development_expense_growth: Option<f64>,
    #[serde(rename = "sgaexpensesGrowth")]
    pub selling_general_and_administrative_expenses_growth: Option<f64>,
    pub free_cash_flow_growth: Option<f64>,
    pub ten_y_revenue_growth_per_share: Option<f64>,
    pub five_y_revenue_growth_per_share: Option<f64>,
    pub three_y_revenue_growth_per_share: Option<f64>,
    #[serde(rename = "tenYOperatingCFGrowthPerShare")]
    pub ten_y_operating_cf_growth_per_share: Option<f64>,
    #[serde(rename = "fiveYOperatingCFGrowthPerShare")]
    pub five_y_operating_cf_growth_per_share: Option<f64>,
    #[serde(rename = "threeYOperatingCFGrowthPerShare")]
    pub three_y_operating_cf_growth_per_share: Option<f64>,
    pub ten_y_net_income_growth_per_share: Option<f64>,
    pub five_y_net_income_growth_per_share: Option<f64>,
    pub three_y_net_income_growth_per_share: Option<f64>,
    pub ten_y_shareholders_equity_growth_per_share: Option<f64>,
    pub five_y_shareholders_equity_growth_per_share: Option<f64>,
    pub three_y_shareholders_equity_growth_per_share: Option<f64>,
    #[serde(rename = "tenYDividendperShareGrowthPerShare")]
    pub ten_y_dividend_per_share_growth_per_share: Option<f64>,
    #[serde(rename = "fiveYDividendperShareGrowthPerShare")]
    pub five_y_dividend_per_share_growth_per_share: Option<f64>,
    #[serde(rename = "threeYDividendperShareGrowthPerShare")]
    pub three_y_dividend_per_share_growth_per_share: Option<f64>,
    pub ebitda_growth: Option<f64>,
    pub growth_capital_expenditure: Option<f64>,
    pub ten_y_bottom_line_net_income_growth_per_share: Option<f64>,
    pub five_y_bottom_line_net_income_growth_per_share: Option<f64>,
    pub three_y_bottom_line_net_income_growth_per_share: Option<f64>,
}
