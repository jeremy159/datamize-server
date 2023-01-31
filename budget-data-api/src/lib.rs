pub mod config;
mod data_builder;
mod error;
pub mod web_scraper;

use crate::config::BudgetDataConfig;
pub use data_builder::types::CategoryIdToNameMap;
pub use data_builder::utils::get_subtransactions_category_ids;
pub use error::Error;
use ynab::types::{Category, ScheduledTransactionDetail};

pub type Result<T> = std::result::Result<T, Error>;
pub type BudgetDetails = data_builder::types::BudgetDetails;
pub type CommonExpanseEstimationPerPerson = data_builder::types::CommonExpanseEstimationPerPerson;
pub type ScheduledTransactionsDistribution = data_builder::types::ScheduledTransactionsDistribution;

pub fn build_budget_details(
    categories: &[Category],
    scheduled_transactions: &[ScheduledTransactionDetail],
) -> Result<BudgetDetails> {
    let config = BudgetDataConfig::build();

    data_builder::budget_details(
        categories,
        scheduled_transactions,
        &config.budget_calculation_data,
    )
}

pub fn build_budget_summary(
    categories: &[Category],
    scheduled_transactions: &[ScheduledTransactionDetail],
) -> Result<Vec<CommonExpanseEstimationPerPerson>> {
    let budget_details = build_budget_details(categories, scheduled_transactions)?;

    data_builder::common_expanses(&budget_details, scheduled_transactions)
}

pub fn build_scheduled_transactions(
    scheduled_transactions: &[ScheduledTransactionDetail],
    category_id_to_name_map: &CategoryIdToNameMap,
) -> Result<ScheduledTransactionsDistribution> {
    data_builder::scheduled_transactions(scheduled_transactions, category_id_to_name_map)
}
