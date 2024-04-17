use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Individual budgeter
mod i_budgeter;
// Total budgeter
mod t_budgeter;

pub use i_budgeter::*;
pub use t_budgeter::*;

pub trait BudgeterExt {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn payee_ids(&self) -> &[Uuid];

    fn salary_month(&self) -> i64 {
        Default::default()
    }

    fn proportion(&self) -> f64 {
        Default::default()
    }

    fn common_expenses(&self) -> i64 {
        Default::default()
    }

    fn individual_expenses(&self) -> i64 {
        Default::default()
    }

    fn left_over(&self) -> i64 {
        Default::default()
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Empty;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Configured {
    id: Uuid,
    /// Name of the Person to use.
    name: String,
    /// All payee ids related to this person. Typically some Inflow payees.
    payee_ids: Vec<Uuid>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SalaryFragment {
    /// Name of the payee if defined.
    pub payee_name: Option<String>,
    /// Amount of this salary.
    pub payee_amount: i64,
    /// Dates when this salary fragment is repeated throughout the month.
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub occurrences: Vec<NaiveDate>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ComputedSalary {
    #[serde(flatten)]
    configured: Configured,
    /// Total salary inflow for this month. This number can vary from one month to the other.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..10000000"))]
    salary_month: i64,
    /// Gives a breakdown of what is composing the salary for this month.
    fragmented_salary: HashMap<Uuid, Vec<SalaryFragment>>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ComputedExpenses {
    #[serde(flatten)]
    compuded_salary: ComputedSalary,
    /// The proportion to be paid on the common expenses.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0.0..1.0"))]
    proportion: f64,
    /// The common expenses of this budgeter for this month.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    common_expenses: i64,
    /// The individual expenses of this budgeter for this month.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..100000"))]
    individual_expenses: i64,
    /// The left over amount for this budgeter.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..10000"))]
    left_over: i64,
}

pub trait BudgeterState {}
impl BudgeterState for Empty {}
impl BudgeterState for Configured {}
impl BudgeterState for ComputedSalary {}
impl BudgeterState for ComputedExpenses {}
