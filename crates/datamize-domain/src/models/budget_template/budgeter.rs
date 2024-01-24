use std::collections::HashMap;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{expense, BudgeterConfig, DatamizeScheduledTransaction, Expense};

pub trait BudgeterExt {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn payee_ids(&self) -> &[Uuid];

    fn salary(&self) -> i64 {
        Default::default()
    }

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

/// A Budgeter represents someone that has income and expenses for the month.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Budgeter<S: BudgeterState> {
    #[serde(flatten)]
    extra: S,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TotalBudgeter<S: BudgeterState> {
    #[serde(flatten)]
    extra: S,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
        scheduled_transactions: &[DatamizeScheduledTransaction],
        date: &DateTime<Local>,
    ) -> Budgeter<ComputedSalary> {
        let mut fragmented_salary = HashMap::new();
        // TODO: Handle if transaction has sub transactions, e.g. DTI which included rrsp and health insurance...
        scheduled_transactions
            .iter()
            .filter(|st| match &st.payee_id {
                Some(ref id) => self.extra.payee_ids.contains(id),
                None => false,
            })
            .for_each(|st| {
                let repeats = st.get_dates_when_transaction_repeats(date).len();
                let salary_fragment = SalaryFragment {
                    payee_name: st.payee_name.clone(),
                    payee_amount: st.amount,
                    repeats: if repeats > 0 { repeats } else { 1 },
                };
                let entry = fragmented_salary
                    .entry(st.payee_id.unwrap()) // We know here payee_id is defined
                    .or_insert_with(|| Vec::with_capacity(1));
                entry.push(salary_fragment);
            });

        let salary = fragmented_salary
            .values()
            .flatten()
            .map(|fs| fs.payee_amount)
            .sum();
        let salary_month = fragmented_salary
            .values()
            .flatten()
            .map(|fs| fs.payee_amount * fs.repeats as i64)
            .sum();

        Budgeter {
            extra: ComputedSalary {
                salary,
                salary_month,
                configured: self.extra,
                fragmented_salary,
            },
        }
    }
}

impl BudgeterExt for Budgeter<Configured> {
    fn id(&self) -> Uuid {
        self.extra.id
    }

    fn name(&self) -> &str {
        &self.extra.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.payee_ids
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
                fragmented_salary: HashMap::new(),
            },
        }
    }
}

impl BudgeterExt for TotalBudgeter<Configured> {
    fn id(&self) -> Uuid {
        self.extra.id
    }

    fn name(&self) -> &str {
        &self.extra.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.payee_ids
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SalaryFragment {
    payee_name: Option<String>,
    payee_amount: i64,
    repeats: usize,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ComputedSalary {
    #[serde(flatten)]
    configured: Configured,
    /// Single occurence salary amount.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..1000000"))]
    salary: i64,
    /// Total salary inflow for this month. This number can vary from one month to the other.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0..10000000"))]
    salary_month: i64,
    fragmented_salary: HashMap<Uuid, Vec<SalaryFragment>>,
}

impl Budgeter<ComputedSalary> {
    pub fn compute_expenses(
        self,
        total_budgeter: &TotalBudgeter<ComputedExpenses>,
        expenses: &[&Expense<expense::Computed>],
    ) -> Budgeter<ComputedExpenses> {
        let proportion = if total_budgeter.salary_month() == 0 {
            0.0
        } else {
            self.extra.salary_month as f64 / total_budgeter.salary_month() as f64
        };
        let common_expenses = (proportion * total_budgeter.extra.common_expenses as f64) as i64;
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

impl BudgeterExt for Budgeter<ComputedSalary> {
    fn id(&self) -> Uuid {
        self.extra.configured.id
    }

    fn name(&self) -> &str {
        &self.extra.configured.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.configured.payee_ids
    }

    fn salary(&self) -> i64 {
        self.extra.salary
    }

    fn salary_month(&self) -> i64 {
        self.extra.salary_month
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

impl BudgeterExt for TotalBudgeter<ComputedSalary> {
    fn id(&self) -> Uuid {
        self.extra.configured.id
    }

    fn name(&self) -> &str {
        &self.extra.configured.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.configured.payee_ids
    }

    fn salary(&self) -> i64 {
        self.extra.salary
    }

    fn salary_month(&self) -> i64 {
        self.extra.salary_month
    }
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

impl BudgeterExt for Budgeter<ComputedExpenses> {
    fn id(&self) -> Uuid {
        self.extra.compuded_salary.configured.id
    }

    fn name(&self) -> &str {
        &self.extra.compuded_salary.configured.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.compuded_salary.configured.payee_ids
    }

    fn salary(&self) -> i64 {
        self.extra.compuded_salary.salary
    }

    fn salary_month(&self) -> i64 {
        self.extra.compuded_salary.salary_month
    }

    fn proportion(&self) -> f64 {
        self.extra.proportion
    }

    fn common_expenses(&self) -> i64 {
        self.extra.common_expenses
    }

    fn individual_expenses(&self) -> i64 {
        self.extra.individual_expenses
    }

    fn left_over(&self) -> i64 {
        self.extra.left_over
    }
}

impl BudgeterExt for TotalBudgeter<ComputedExpenses> {
    fn id(&self) -> Uuid {
        self.extra.compuded_salary.configured.id
    }

    fn name(&self) -> &str {
        &self.extra.compuded_salary.configured.name
    }

    fn payee_ids(&self) -> &[Uuid] {
        &self.extra.compuded_salary.configured.payee_ids
    }

    fn salary(&self) -> i64 {
        self.extra.compuded_salary.salary
    }

    fn salary_month(&self) -> i64 {
        self.extra.compuded_salary.salary_month
    }

    fn proportion(&self) -> f64 {
        self.extra.proportion
    }

    fn common_expenses(&self) -> i64 {
        self.extra.common_expenses
    }

    fn individual_expenses(&self) -> i64 {
        self.extra.individual_expenses
    }

    fn left_over(&self) -> i64 {
        self.extra.left_over
    }
}

pub trait BudgeterState {}
impl BudgeterState for Empty {}
impl BudgeterState for Configured {}
impl BudgeterState for ComputedSalary {}
impl BudgeterState for ComputedExpenses {}

#[cfg(any(feature = "testutils", test))]
impl<S: BudgeterState + fake::Dummy<fake::Faker>> fake::Dummy<fake::Faker> for Budgeter<S> {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &fake::Faker, rng: &mut R) -> Self {
        use fake::Fake;
        let extra = config.fake_with_rng(rng);

        Self { extra }
    }
}
