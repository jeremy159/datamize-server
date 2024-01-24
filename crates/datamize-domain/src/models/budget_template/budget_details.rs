use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, Months};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::Category;

use super::{
    expense::Computed, Budgeter, BudgeterExt, ComputedSalary, DatamizeScheduledTransaction,
    Expense, ExpenseCategorization, ExpenseType, PartiallyComputed, Uncomputed,
};

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Default, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MonthTarget {
    Previous,
    #[default]
    Current,
    Next,
}

impl From<MonthTarget> for DateTime<Local> {
    fn from(value: MonthTarget) -> Self {
        match value {
            MonthTarget::Previous => Local::now().checked_sub_months(Months::new(1)).unwrap(),
            MonthTarget::Current => Local::now(),
            MonthTarget::Next => Local::now().checked_add_months(Months::new(1)).unwrap(),
        }
        .with_day(1)
        .unwrap()
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct TemplateParams {
    pub month: Option<MonthTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalMetadata {
    /// Salary related incomes
    pub monthly_income: i64,
    /// Total income, before substracting health insurance and work-related retirement savings
    pub total_monthly_income: i64,
    /// The tartet each expense type should follow. For example, all fixed expenses shouldn't go over 60% of total income.
    pub proportion_target_per_expense_type: HashMap<ExpenseType, f64>,
}

#[cfg(any(feature = "testutils", test))]
impl fake::Dummy<fake::Faker> for GlobalMetadata {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &fake::Faker, rng: &mut R) -> Self {
        use fake::Fake;
        let monthly_income = Fake::fake_with_rng(&(1..10000000), rng);
        let total_monthly_income = Fake::fake_with_rng(&(1..10000000), rng);

        let mut proportion_target_per_expense_type = HashMap::new();
        let len = 5;
        for _ in 0..len {
            proportion_target_per_expense_type.insert(
                config.fake_with_rng(rng),
                Fake::fake_with_rng(&(0.0..1.0), rng),
            );
        }

        Self {
            monthly_income,
            total_monthly_income,
            proportion_target_per_expense_type,
        }
    }
}

impl Default for GlobalMetadata {
    fn default() -> Self {
        let tuples = [
            (ExpenseType::Fixed, 0.6_f64),
            (ExpenseType::Variable, 0.1_f64),
            (ExpenseType::ShortTermSaving, 0.1_f64),
            (ExpenseType::LongTermSaving, 0.1_f64),
            (ExpenseType::RetirementSaving, 0.1_f64),
        ];
        let proportion_target_per_expense_type = tuples.into_iter().collect();

        Self {
            monthly_income: 0,
            total_monthly_income: 0,
            proportion_target_per_expense_type,
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BudgetDetails {
    global: GlobalMetadata,
    expenses: Vec<Expense<Computed>>,
}

impl BudgetDetails {
    pub fn global_metadata(&self) -> &GlobalMetadata {
        &self.global
    }

    pub fn expenses(&self) -> &[Expense<Computed>] {
        &self.expenses
    }

    pub fn build(
        categories: Vec<Category>,
        scheduled_transactions: Vec<DatamizeScheduledTransaction>,
        date: &DateTime<Local>,
        expenses_categorization: Vec<ExpenseCategorization>,
        budgeters: &[Budgeter<ComputedSalary>],
    ) -> Self {
        let mut scheduled_transactions_map =
            build_category_to_scheduled_transaction_map(scheduled_transactions, date);

        let expenses: Vec<Expense<PartiallyComputed>> = categories
            .into_iter()
            .filter(|c| !c.hidden && !c.deleted)
            .map(|c| {
                let cat_id = c.id;
                let mut expense: Expense<Uncomputed> = c.into();
                if let Some(scheduled_transactions) = scheduled_transactions_map.remove(&cat_id) {
                    expense = expense.with_scheduled_transactions(scheduled_transactions);
                }
                expense.compute_amounts()
            })
            .collect::<Vec<_>>();

        let mut expenses: Vec<_> = expenses
            .into_iter()
            .map(|e| {
                e.set_categorization(&expenses_categorization)
                    .set_individual_association(budgeters)
            })
            .filter(|e| e.expense_type() != &ExpenseType::Undefined)
            .collect();

        expenses.sort_by(|first, second| first.sub_expense_type().cmp(second.sub_expense_type()));

        let monthly_income = budgeters.iter().map(|b| b.salary_month()).sum();
        let health_insurance_amount = expenses
            .iter()
            .filter(|e| e.name().contains("Assurance Santé"))
            .map(|e| e.projected_amount())
            .sum::<i64>();
        let retirement_savings_total = expenses
            .iter()
            .filter(|e| e.expense_type() == &ExpenseType::RetirementSaving)
            .map(|e| e.projected_amount())
            .sum::<i64>();

        let total_monthly_income =
            monthly_income + health_insurance_amount + retirement_savings_total;

        let expenses = expenses
            .into_iter()
            .map(|e| e.compute_proportions(total_monthly_income))
            .collect();

        Self {
            global: GlobalMetadata {
                monthly_income,
                total_monthly_income,
                ..Default::default()
            },
            expenses,
        }
    }
}

fn build_category_to_scheduled_transaction_map<T: Into<DatamizeScheduledTransaction>>(
    scheduled_transactions: Vec<T>,
    date: &DateTime<Local>,
) -> HashMap<Uuid, Vec<DatamizeScheduledTransaction>> {
    let scheduled_transactions: Vec<DatamizeScheduledTransaction> =
        scheduled_transactions.into_iter().map(Into::into).collect();

    let scheduled_transactions: Vec<_> = scheduled_transactions
        .into_iter()
        .filter(|dst| !dst.deleted && dst.category_id.is_some())
        .flat_map(|dst| dst.flatten())
        .flat_map(|dst| dst.get_transactions_within_month(date))
        .collect();

    let mut hash_map: HashMap<Uuid, Vec<DatamizeScheduledTransaction>> =
        HashMap::with_capacity(scheduled_transactions.len());

    for dst in scheduled_transactions {
        if let Some(category_id) = dst.category_id {
            hash_map.entry(category_id).or_default().push(dst);
        }
    }

    hash_map
}
