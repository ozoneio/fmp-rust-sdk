//! Financial statements and their reporting metadata.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

/// Reporting identifiers shared by statements, key metrics, ratios, and financial growth.
/// Filing details are absent when the endpoint does not return them.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialPeriod {
    #[serde(deserialize_with = "super::non_empty_string")]
    pub symbol: String,
    pub date: NaiveDate,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub fiscal_year: String,
    /// Provider reporting label, such as FY, Q1, Q2, Q3, or Q4.
    #[serde(deserialize_with = "super::non_empty_string")]
    pub period: String,
    #[serde(deserialize_with = "super::non_empty_string")]
    pub reported_currency: String,
    pub cik: Option<String>,
    #[serde(default, deserialize_with = "super::optional_date")]
    pub filing_date: Option<NaiveDate>,
    #[serde(default, with = "super::optional_timestamp")]
    pub accepted_date: Option<NaiveDateTime>,
}

/// Income statement values from `income-statement` and `income-statement-ttm`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatement {
    #[serde(flatten)]
    pub metadata: FinancialPeriod,
    pub revenue: Option<f64>,
    pub cost_of_revenue: Option<f64>,
    pub gross_profit: Option<f64>,
    pub research_and_development_expenses: Option<f64>,
    pub general_and_administrative_expenses: Option<f64>,
    pub selling_and_marketing_expenses: Option<f64>,
    pub selling_general_and_administrative_expenses: Option<f64>,
    pub other_expenses: Option<f64>,
    pub operating_expenses: Option<f64>,
    pub cost_and_expenses: Option<f64>,
    pub net_interest_income: Option<f64>,
    pub interest_income: Option<f64>,
    pub interest_expense: Option<f64>,
    pub depreciation_and_amortization: Option<f64>,
    pub ebitda: Option<f64>,
    pub ebit: Option<f64>,
    pub non_operating_income_excluding_interest: Option<f64>,
    pub operating_income: Option<f64>,
    pub total_other_income_expenses_net: Option<f64>,
    pub income_before_tax: Option<f64>,
    pub income_tax_expense: Option<f64>,
    pub net_income_from_continuing_operations: Option<f64>,
    pub net_income_from_discontinued_operations: Option<f64>,
    pub other_adjustments_to_net_income: Option<f64>,
    pub net_income: Option<f64>,
    pub net_income_deductions: Option<f64>,
    pub bottom_line_net_income: Option<f64>,
    pub eps: Option<f64>,
    pub eps_diluted: Option<f64>,
    pub weighted_average_shs_out: Option<f64>,
    pub weighted_average_shs_out_dil: Option<f64>,
}

/// Balance sheet values from `balance-sheet-statement` and its TTM endpoint.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheetStatement {
    #[serde(flatten)]
    pub metadata: FinancialPeriod,
    pub cash_and_cash_equivalents: Option<f64>,
    pub short_term_investments: Option<f64>,
    pub cash_and_short_term_investments: Option<f64>,
    pub net_receivables: Option<f64>,
    pub accounts_receivables: Option<f64>,
    pub other_receivables: Option<f64>,
    pub inventory: Option<f64>,
    pub prepaids: Option<f64>,
    pub other_current_assets: Option<f64>,
    pub total_current_assets: Option<f64>,
    pub property_plant_equipment_net: Option<f64>,
    pub goodwill: Option<f64>,
    pub intangible_assets: Option<f64>,
    pub goodwill_and_intangible_assets: Option<f64>,
    pub long_term_investments: Option<f64>,
    pub tax_assets: Option<f64>,
    pub other_non_current_assets: Option<f64>,
    pub total_non_current_assets: Option<f64>,
    pub other_assets: Option<f64>,
    pub total_assets: Option<f64>,
    pub total_payables: Option<f64>,
    pub account_payables: Option<f64>,
    pub other_payables: Option<f64>,
    pub accrued_expenses: Option<f64>,
    pub short_term_debt: Option<f64>,
    pub capital_lease_obligations_current: Option<f64>,
    pub tax_payables: Option<f64>,
    pub deferred_revenue: Option<f64>,
    pub other_current_liabilities: Option<f64>,
    pub total_current_liabilities: Option<f64>,
    pub long_term_debt: Option<f64>,
    pub capital_lease_obligations_non_current: Option<f64>,
    pub deferred_revenue_non_current: Option<f64>,
    pub deferred_tax_liabilities_non_current: Option<f64>,
    pub other_non_current_liabilities: Option<f64>,
    pub total_non_current_liabilities: Option<f64>,
    pub other_liabilities: Option<f64>,
    pub capital_lease_obligations: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub treasury_stock: Option<f64>,
    pub preferred_stock: Option<f64>,
    pub common_stock: Option<f64>,
    pub retained_earnings: Option<f64>,
    pub additional_paid_in_capital: Option<f64>,
    pub accumulated_other_comprehensive_income_loss: Option<f64>,
    pub other_total_stockholders_equity: Option<f64>,
    pub total_stockholders_equity: Option<f64>,
    pub total_equity: Option<f64>,
    pub minority_interest: Option<f64>,
    pub total_liabilities_and_total_equity: Option<f64>,
    pub total_investments: Option<f64>,
    pub total_debt: Option<f64>,
    pub net_debt: Option<f64>,
}

/// Cash flow values from `cash-flow-statement` and its TTM endpoint.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlowStatement {
    #[serde(flatten)]
    pub metadata: FinancialPeriod,
    pub net_income: Option<f64>,
    pub depreciation_and_amortization: Option<f64>,
    pub deferred_income_tax: Option<f64>,
    pub stock_based_compensation: Option<f64>,
    pub change_in_working_capital: Option<f64>,
    pub accounts_receivables: Option<f64>,
    pub inventory: Option<f64>,
    pub accounts_payables: Option<f64>,
    pub other_working_capital: Option<f64>,
    pub other_non_cash_items: Option<f64>,
    pub net_cash_provided_by_operating_activities: Option<f64>,
    pub investments_in_property_plant_and_equipment: Option<f64>,
    pub acquisitions_net: Option<f64>,
    pub purchases_of_investments: Option<f64>,
    pub sales_maturities_of_investments: Option<f64>,
    pub other_investing_activities: Option<f64>,
    pub net_cash_provided_by_investing_activities: Option<f64>,
    pub net_debt_issuance: Option<f64>,
    pub long_term_net_debt_issuance: Option<f64>,
    pub short_term_net_debt_issuance: Option<f64>,
    pub net_stock_issuance: Option<f64>,
    pub net_common_stock_issuance: Option<f64>,
    pub common_stock_issuance: Option<f64>,
    pub common_stock_repurchased: Option<f64>,
    pub net_preferred_stock_issuance: Option<f64>,
    pub net_dividends_paid: Option<f64>,
    pub common_dividends_paid: Option<f64>,
    pub preferred_dividends_paid: Option<f64>,
    pub other_financing_activities: Option<f64>,
    pub net_cash_provided_by_financing_activities: Option<f64>,
    pub effect_of_forex_changes_on_cash: Option<f64>,
    pub net_change_in_cash: Option<f64>,
    pub cash_at_end_of_period: Option<f64>,
    pub cash_at_beginning_of_period: Option<f64>,
    pub operating_cash_flow: Option<f64>,
    pub capital_expenditure: Option<f64>,
    pub free_cash_flow: Option<f64>,
    pub income_taxes_paid: Option<f64>,
    pub interest_paid: Option<f64>,
}
