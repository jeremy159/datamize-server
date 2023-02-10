use serde::Serialize;

use super::FinancialResource;

#[derive(Debug, Serialize)]
pub struct TotalSummary {
    /// The total amount of the current section.
    pub total: i64,
    /// The percentage of variation compared to the previous month's section.
    pub percent_variation: f64,
    /// The money balance of variation compared to the previous month's section.
    pub balance_variation: i64,
}

/// A balance sheet of the month.
#[derive(Debug, Serialize)]
pub struct Month {
    /// The month in ISO format, e.g. "2020-12-25".
    pub month: chrono::Month,
    /// Net Assets summary section. Includes the variation with the previous month.
    /// Net Assets is the total of owned assets minus the total of liabilities.
    pub net_assets: TotalSummary,
    /// Net Portfolio summary section. Includes the variation with the previous month.
    /// Net Portfolio is the total of owned assets that are tangible cash. For example, bank or investments accounts
    /// are tangible cash assets but not the value of your house or car.
    pub net_portfolio: TotalSummary,
    /// All of the Assets and Liabilities of the current month are regrouped here.
    pub resources: Vec<FinancialResource>,
}
