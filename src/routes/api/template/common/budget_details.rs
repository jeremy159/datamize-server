use std::collections::HashMap;

use uuid::Uuid;
use ynab::types::{Category, ScheduledTransactionDetail};

use crate::{
    config::{BugdetCalculationDataSettings, PersonSalarySettings},
    models::budget_template::{BudgetDetails, Expense, ExpenseType},
};

use super::utils;

/// Gives bugdet details, spliting expenses by their type and sub-type.
pub fn build_budget_details(
    categories: Vec<Category>,
    scheduled_transactions: Vec<ScheduledTransactionDetail>,
    budget_calculation_data_settings: &BugdetCalculationDataSettings,
    person_salary_settings: &[PersonSalarySettings],
) -> anyhow::Result<BudgetDetails> {
    let salary_per_person =
        utils::get_salary_per_person_per_month(&scheduled_transactions, person_salary_settings);

    let scheduled_transactions_map =
        build_category_to_scheduled_transaction_map(scheduled_transactions);

    let mut output: BudgetDetails = BudgetDetails::default();
    output.expenses.extend(
        categories
            .into_iter()
            .filter(|c| !c.hidden && !c.deleted)
            .map(|c| {
                let expense: Expense = c.into();
                expense
                    .set_projected_amount(&scheduled_transactions_map)
                    .set_current_amount(&scheduled_transactions_map)
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
) -> HashMap<Uuid, Vec<ScheduledTransactionDetail>> {
    let mut hash_map: HashMap<Uuid, Vec<ScheduledTransactionDetail>> =
        HashMap::with_capacity(scheduled_transactions.len());

    let mut scheduled_transactions_filtered: Vec<ScheduledTransactionDetail> = vec![];
    let mut repeated_sts: Vec<ScheduledTransactionDetail> = vec![];

    scheduled_transactions_filtered.extend(
        scheduled_transactions
            .into_iter()
            .filter(|st| st.category_id.is_some())
            .filter(|st| !st.deleted)
            .filter(|st| utils::is_transaction_in_next_30_days(&st.date_next))
            .map(|st| {
                repeated_sts.extend(utils::find_repeatable_transactions(&st));
                st
            }),
    );

    scheduled_transactions_filtered.extend(repeated_sts);

    for st in scheduled_transactions_filtered {
        let non_deleted_sub_st: Vec<_> = st
            .subtransactions
            .iter()
            .filter(|sub_st| !sub_st.deleted)
            .collect();

        if !non_deleted_sub_st.is_empty() {
            for sub_st in non_deleted_sub_st {
                let transformed_st: ScheduledTransactionDetail =
                    st.clone().from_subtransaction(sub_st);

                let category_id = sub_st.category_id.unwrap();

                let entry = hash_map.entry(category_id);
                entry.or_insert_with(Vec::new).push(transformed_st);
            }
        } else {
            let category_id = st.category_id.unwrap();
            let entry = hash_map.entry(category_id);
            entry.or_insert_with(Vec::new).push(st);
        }
    }

    hash_map
}
