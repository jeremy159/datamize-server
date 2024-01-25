use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::budget_template::expense, Budgeter, BudgeterExt, BudgeterState, ComputedExpenses,
    ComputedSalary, Configured, Empty, Expense,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TotalBudgeter<S: BudgeterState> {
    #[serde(flatten)]
    extra: S,
}

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

impl TotalBudgeter<Configured> {
    pub fn compute_salary(
        self,
        budgeters: &[Budgeter<ComputedSalary>],
    ) -> TotalBudgeter<ComputedSalary> {
        TotalBudgeter {
            extra: ComputedSalary {
                salary: budgeters.iter().map(|b| b.salary()).sum(),
                salary_month: budgeters.iter().map(|b| b.salary_month()).sum(),
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
            .filter(
                |&e| match budgeters.iter().find(|b| e.name().contains(b.name())) {
                    Some(_) => {
                        individual_expenses.push(e);
                        false
                    }
                    None => true,
                },
            )
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
