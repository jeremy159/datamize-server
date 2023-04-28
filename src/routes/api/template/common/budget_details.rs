use std::collections::HashMap;

use chrono::{DateTime, Local};
use uuid::Uuid;
use ynab::types::{Category, ScheduledTransactionDetail};

use crate::{
    config::{BugdetCalculationDataSettings, PersonSalarySettings},
    models::budget_template::{BudgetDetails, Expense, ExpenseType},
};

use super::{flatten_sub_transactions, get_scheduled_transactions_within_month, utils};

/// Gives bugdet details, spliting expenses by their type and sub-type.
pub fn build_budget_details(
    categories: Vec<Category>,
    scheduled_transactions: Vec<ScheduledTransactionDetail>,
    date: &DateTime<Local>,
    budget_calculation_data_settings: &BugdetCalculationDataSettings,
    person_salary_settings: &[PersonSalarySettings],
) -> anyhow::Result<BudgetDetails> {
    let salary_per_person =
        utils::get_salary_per_person_per_month(&scheduled_transactions, person_salary_settings);

    let mut scheduled_transactions_map =
        build_category_to_scheduled_transaction_map(scheduled_transactions, date);

    let mut output: BudgetDetails = BudgetDetails::default();
    output.expenses.extend(
        categories
            .into_iter()
            .filter(|c| !c.hidden && !c.deleted)
            .map(|c| {
                let cat_id = c.id;
                let mut expense: Expense = c.into();
                if let Some(scheduled_transactions) = scheduled_transactions_map.remove(&cat_id) {
                    expense = expense.with_scheduled_transactions(scheduled_transactions);
                }
                expense
                    .compute_projected_amount()
                    .compute_current_amount()
                    .set_categorization(budget_calculation_data_settings)
                    .set_individual_association(person_salary_settings)
            })
            .collect::<Vec<_>>(),
    );

    output
        .expenses
        .sort_by(|first, second| first.sub_expense_type.cmp(&second.sub_expense_type));

    // Filter out undefined
    output
        .expenses
        .retain(|e| e.expense_type != ExpenseType::Undefined);

    output.expenses.extend(
        budget_calculation_data_settings
            .external_expenses
            .iter()
            .cloned()
            .map(Expense::from),
    );

    // Compute monthly income total
    output.global.monthly_income = salary_per_person.iter().map(|sp| sp.salary_per_month).sum();
    let health_insurance_amount = match output
        .expenses
        .iter()
        .find(|c| c.name.contains("Assurance Santé"))
    {
        Some(c) => c.projected_amount,
        None => 0,
    };
    let retirement_savings_total = output
        .expenses
        .iter()
        .filter(|e| e.expense_type == ExpenseType::RetirementSaving)
        .map(|c| c.projected_amount)
        .sum::<i64>();

    output.global.total_monthly_income =
        output.global.monthly_income + health_insurance_amount + retirement_savings_total;

    // Compute proportion for each expense
    output.expenses.iter_mut().for_each(|e| {
        e.projected_proportion =
            e.projected_amount as f64 / output.global.total_monthly_income as f64;
        e.current_proportion = e.current_amount as f64 / output.global.total_monthly_income as f64;
    });
    Ok(output)
}

fn build_category_to_scheduled_transaction_map(
    scheduled_transactions: Vec<ScheduledTransactionDetail>,
    date: &DateTime<Local>,
) -> HashMap<Uuid, Vec<ScheduledTransactionDetail>> {
    let mut hash_map: HashMap<Uuid, Vec<ScheduledTransactionDetail>> =
        HashMap::with_capacity(scheduled_transactions.len());

    let scheduled_transactions_filtered: Vec<ScheduledTransactionDetail> = scheduled_transactions
        .into_iter()
        .filter(|st| st.category_id.is_some())
        .filter(|st| !st.deleted)
        .flat_map(|st| get_scheduled_transactions_within_month(&st, date))
        .collect();

    for st in flatten_sub_transactions(scheduled_transactions_filtered) {
        if let Some(category_id) = st.category_id {
            hash_map
                .entry(category_id)
                .or_insert_with(Vec::new)
                .push(st);
        }
    }

    hash_map
}
