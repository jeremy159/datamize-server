use serde::{Deserialize, Serialize};

use crate::Category;

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthSummary {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthSummary {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthSummaryDelta {
    pub months: Vec<MonthSummary>,
    pub server_knowledge: i64,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthDetail {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
    pub categories: Vec<Category>,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthDetail {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
    pub categories: Vec<Category>,
}
