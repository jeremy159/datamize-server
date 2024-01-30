use serde::{Deserialize, Serialize};

use crate::Category;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthSummary {
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub month: chrono::NaiveDate,
    pub note: Option<String>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub income: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub budgeted: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub activity: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub to_be_budgeted: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100"))]
    pub age_of_money: Option<i32>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthSummaryDelta {
    pub months: Vec<MonthSummary>,
    pub server_knowledge: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthDetail {
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub month: chrono::NaiveDate,
    pub note: Option<String>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub income: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub budgeted: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub activity: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub to_be_budgeted: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100"))]
    pub age_of_money: Option<i32>,
    pub deleted: bool,
    pub categories: Vec<Category>,
}
