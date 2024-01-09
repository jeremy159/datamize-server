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
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    pub budgeted: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub activity: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    pub balance: i64,
    pub goal_type: Option<GoalType>,
    /// A day offset modifier for the goal's due date. When goal_cadence is 2 (Weekly),
    /// this value specifies which day of the week the goal is due (0 = Sunday, 6 = Saturday).
    /// Otherwise, this value specifies which day of the month the goal is due
    /// (1 = 1st, 31 = 31st, null = Last day of Month).
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..32"))]
    pub goal_day: Option<i32>,
    /// The goal cadence. Value in range 0-14. There are two subsets of these values which behave differently.
    /// For values 0, 1, 2, and 13, the goal's due date repeats every goal_cadence * goal_cadence_frequency,
    /// where 0 = None, 1 = Monthly, 2 = Weekly, and 13 = Yearly. For example,
    /// goal_cadence 1 with goal_cadence_frequency 2 means the goal is due every other month.
    /// For values 3-12 and 14, goal_cadence_frequency is ignored and the goal's due date repeats every goal_cadence,
    /// where 3 = Every 2 Months, 4 = Every 3 Months, ..., 12 = Every 11 Months, and 14 = Every 2 Years.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..15"))]
    pub goal_cadence: Option<i32>,
    /// The goal cadence frequency. When goal_cadence is 0, 1, 2, or 13, a goal's due date
    /// repeats every goal_cadence * goal_cadence_frequency. For example, goal_cadence 1 with
    /// goal_cadence_frequency 2 means the goal is due every other month. When goal_cadence is
    /// 3-12 or 14, goal_cadence_frequency is ignored.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "1..14"))]
    pub goal_cadence_frequency: Option<i32>,
    pub goal_creation_month: Option<chrono::NaiveDate>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    pub goal_target: i64,
    pub goal_target_month: Option<chrono::NaiveDate>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100"))]
    pub goal_percentage_complete: Option<i32>,
    /// The number of months, including the current month, left in the current goal period.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100"))]
    pub goal_months_to_budget: Option<i32>,
    /// The amount of funding still needed in the current month to stay on track towards
    /// completing the goal within the current goal period. This amount will generally
    /// correspond to the 'Underfunded' amount in the web and mobile clients except
    /// when viewing a category with a Needed for Spending Goal in a future month.
    /// The web and mobile clients will ignore any funding from a prior goal period when
    /// viewing category with a Needed for Spending Goal in a future month.
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
