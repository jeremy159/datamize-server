use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Categories/getCategoryById
pub struct Category {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub category_group_name: String,
    pub name: String,
    pub hidden: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub budgeted: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub activity: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub balance: i64,
    pub goal_type: Option<GoalType>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..32"))]
    pub goal_day: Option<i32>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..15"))]
    pub goal_cadence: Option<i32>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "1..14"))]
    pub goal_cadence_frequency: Option<i32>,
    pub goal_creation_month: Option<chrono::NaiveDate>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub goal_target: i64,
    pub goal_target_month: Option<chrono::NaiveDate>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100"))]
    pub goal_percentage_complete: Option<i32>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100"))]
    pub goal_months_to_budget: Option<i32>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    pub goal_under_funded: Option<i64>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    pub goal_overall_funded: Option<i64>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    pub goal_overall_left: Option<i64>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(type_name = "goal_type"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalType {
    #[serde(rename = "TB")]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(rename = "TB"))]
    TargetBalance,
    #[serde(rename = "TBD")]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(rename = "TBD"))]
    TargetBalanceByDate,
    #[serde(rename = "MF")]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(rename = "MF"))]
    MonthlyFunding,
    #[serde(rename = "NEED")]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(rename = "NEED"))]
    PlanYourSpending,
    #[serde(rename = "DEBT")]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(rename = "DEBT"))]
    Debt,
}

impl fmt::Display for GoalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GoalType::TargetBalance => write!(f, "TB"),
            GoalType::TargetBalanceByDate => write!(f, "TBD"),
            GoalType::MonthlyFunding => write!(f, "MF"),
            GoalType::PlanYourSpending => write!(f, "NEED"),
            GoalType::Debt => write!(f, "DEBT"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseGoalTypeError;

impl FromStr for GoalType {
    type Err = ParseGoalTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TB" => Ok(Self::TargetBalance),
            "TBD" => Ok(Self::TargetBalanceByDate),
            "MF" => Ok(Self::MonthlyFunding),
            "NEED" => Ok(Self::PlanYourSpending),
            "DEBT" => Ok(Self::Debt),
            _ => Err(ParseGoalTypeError),
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Categories/updateMonthCategory
pub struct SaveMonthCategory {
    pub budgeted: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryGroup {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryGroupWithCategories {
    pub id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub deleted: bool,
    pub categories: Vec<Category>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroupWithCategoriesDelta {
    pub category_groups: Vec<CategoryGroupWithCategories>,
    pub server_knowledge: i64,
}
