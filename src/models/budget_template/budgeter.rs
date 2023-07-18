use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::ScheduledTransactionDetail;

use super::{expense, find_repeatable_transactions, Expense};

/// A Budgeter represents someone that has income and expenses for the month.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Budgeter<S: BudgeterState> {
    #[serde(flatten)]
    extra: S,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TotalBudgeter<S: BudgeterState> {
    #[serde(flatten)]
    extra: S,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Empty;

impl TotalBudgeter<Empty> {
    pub fn new() -> TotalBudgeter<Configured> {
        TotalBudgeter {
            extra: Configured {
                id: Uuid::new_v4(),
                name: "Total".into(),
                payee_ids: vec![],
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Configured {
    id: Uuid,
    /// Name of the Person to use.
    name: String,
    /// All payee ids related to this person. Typically some Inflow payees.
    payee_ids: Vec<Uuid>,
}

impl From<BudgeterConfig> for Budgeter<Configured> {
    fn from(value: BudgeterConfig) -> Self {
        Budgeter {
            extra: Configured {
                id: value.id,
                name: value.name,
                payee_ids: value.payee_ids,
            },
        }
    }
}

impl Budgeter<Configured> {
    pub fn compute_salary(
        self,
        scheduled_transactions: &[ScheduledTransactionDetail],
    ) -> Budgeter<ComputedSalary> {
        let mut salary = 0;
        let salary_month = scheduled_transactions
            .iter()
            .filter(|st| match &st.payee_id {
                Some(ref id) => self.extra.payee_ids.contains(id),
                None => false,
            })
            .map(|st| {
                salary = st.amount;
                st.amount
                    + find_repeatable_transactions(st)
                        .iter()
                        .map(|v| v.amount)
                        .sum::<i64>()
            })
            .sum();

        Budgeter {
            extra: ComputedSalary {
                salary,
                salary_month,
                configured: self.extra,
            },
        }
    }
}

impl TotalBudgeter<Configured> {
    pub fn compute_salary(
        self,
        budgeters: &[Budgeter<ComputedSalary>],
    ) -> TotalBudgeter<ComputedSalary> {
        TotalBudgeter {
            extra: ComputedSalary {
                salary: budgeters.iter().map(|b| b.extra.salary).sum(),
                salary_month: budgeters.iter().map(|b| b.extra.salary_month).sum(),
                configured: self.extra,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputedSalary {
    #[serde(flatten)]
    configured: Configured,
    /// Single occurence salary amount.
    salary: i64,
    /// Number of times the salary gets repeated for this month. This number can vary from one month to
    /// the other.
    // salary_occurence: i16, // TODO: Check if still needed...
    /// Total salary inflow for this month. This number can vary from one month to the other.
    salary_month: i64,
}

impl Budgeter<ComputedSalary> {
    pub fn name(&self) -> &String {
        // TODO: Maybe define in a Trait?
        &self.extra.configured.name
    }

    pub fn salary_month(&self) -> i64 {
        self.extra.salary_month
    }

    pub fn compute_expenses(
        self,
        total_budgeter: &TotalBudgeter<ComputedExpenses>,
        expenses: &[&Expense<expense::Computed>],
    ) -> Budgeter<ComputedExpenses> {
        let proportion = self.extra.salary_month as f64
            / total_budgeter.extra.compuded_salary.salary_month as f64;
        let common_expenses = ((proportion * (total_budgeter.extra.common_expenses as f64)
            / 1000_f64)
            * 1000_f64) as i64;
        let individual_expenses = expenses
            .iter()
            .filter(|e| e.name().contains(&self.extra.configured.name))
            .map(|e| e.projected_amount())
            .sum();
        let left_over = self.extra.salary_month - common_expenses - individual_expenses;

        Budgeter {
            extra: ComputedExpenses {
                common_expenses,
                proportion,
                individual_expenses,
                left_over,
                compuded_salary: self.extra,
            },
        }
    }
}

impl TotalBudgeter<ComputedSalary> {
    pub fn compute_expenses<'a>(
        self,
        expenses: &'a [Expense<expense::Computed>],
        budgeters: &[Budgeter<ComputedSalary>],
    ) -> (
        TotalBudgeter<ComputedExpenses>,
        Vec<&'a Expense<expense::Computed>>,
    ) {
        let mut individual_expenses: Vec<&Expense<expense::Computed>> = vec![];

        let common_expenses = expenses
            .iter()
            .filter(|&e| !e.is_external())
            .filter(|&e| {
                match budgeters
                    .iter()
                    .find(|b| e.name().contains(&b.extra.configured.name))
                {
                    Some(_) => {
                        individual_expenses.push(e);
                        false
                    }
                    None => true,
                }
            })
            .map(|e| e.projected_amount())
            .sum();

        let total_individual_expenses = individual_expenses
            .iter()
            .map(|ie| ie.projected_amount())
            .sum();
        let left_over = self.extra.salary_month - common_expenses - total_individual_expenses;

        (
            TotalBudgeter {
                extra: ComputedExpenses {
                    common_expenses,
                    proportion: 1.0,
                    individual_expenses: total_individual_expenses,
                    left_over,
                    compuded_salary: self.extra,
                },
            },
            individual_expenses,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputedExpenses {
    #[serde(flatten)]
    compuded_salary: ComputedSalary,
    /// The proportion to be paid on the common expenses.
    proportion: f64,
    /// The common expenses of this budgeter for this month.
    common_expenses: i64,
    /// The individual expenses of this budgeter for this month.
    individual_expenses: i64,
    /// The left over amount for this budgeter.
    left_over: i64,
}

pub trait BudgeterState {}
impl BudgeterState for Empty {}
impl BudgeterState for Configured {}
impl BudgeterState for ComputedSalary {}
impl BudgeterState for ComputedExpenses {}

// TODO: Use this as base state for the Budgeter.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BudgeterConfig {
    pub id: Uuid,
    pub name: String,
    pub payee_ids: Vec<Uuid>,
}

impl BudgeterConfig {
    pub fn new(name: String, payee_ids: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            payee_ids,
        }
    }
}

impl From<SaveBudgeterConfig> for BudgeterConfig {
    fn from(value: SaveBudgeterConfig) -> Self {
        Self::new(value.name, value.payee_ids)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveBudgeterConfig {
    pub name: String,
    pub payee_ids: Vec<Uuid>,
}
