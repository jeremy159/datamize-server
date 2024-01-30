use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    Account, Category, CategoryGroup, MonthDetail, Payee, PayeeLocation,
    ScheduledTransactionSummary, SubTransaction, TransactionSummary,
};

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    pub example_format: String,
    pub decimal_digits: i64,
    pub decimal_separator: String,
    pub symbol_first: bool,
    pub group_separator: String,
    pub currency_symbol: String,
    pub display_symbol: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateFormat {
    pub format: String,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseBudgetSumary {
    pub id: Uuid,
    pub name: String,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub last_modified_on: DateTime<Utc>,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub first_month: NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub last_month: NaiveDate,
    pub date_format: Option<DateFormat>,
    pub currency_format: Option<CurrencyFormat>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct BudgetSummary {
    #[serde(flatten)]
    pub base: BaseBudgetSumary,
}

impl AsRef<BaseBudgetSumary> for BudgetSummary {
    fn as_ref(&self) -> &BaseBudgetSumary {
        &self.base
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct BudgetSummaryWithAccounts {
    #[serde(flatten)]
    pub base: BaseBudgetSumary,
    pub accounts: Vec<Account>,
}

impl AsRef<BaseBudgetSumary> for BudgetSummaryWithAccounts {
    fn as_ref(&self) -> &BaseBudgetSumary {
        &self.base
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct BudgetDetail {
    pub id: Uuid,
    pub name: String,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub last_modified_on: DateTime<Utc>,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub first_month: NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub last_month: NaiveDate,
    pub date_format: Option<DateFormat>,
    pub currency_format: Option<CurrencyFormat>,
    pub accounts: Vec<Account>,
    pub payees: Vec<Payee>,
    pub payee_locations: Vec<PayeeLocation>,
    pub category_groups: Vec<CategoryGroup>,
    pub categories: Vec<Category>,
    pub months: Vec<MonthDetail>,
    pub transactions: Vec<TransactionSummary>,
    pub subtransactions: Vec<SubTransaction>,
    pub scheduled_transactions: Vec<ScheduledTransactionSummary>,
    pub scheduled_subtransactions: Vec<SubTransaction>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDetailDelta {
    pub budget: BudgetDetail,
    pub server_knowledge: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetSettingsById
pub struct BudgetSettings {
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
}
