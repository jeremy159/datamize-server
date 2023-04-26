use std::collections::HashMap;

use super::utils;
use crate::{
    config::PersonSalarySettings,
    models::budget_template::{BudgetDetails, CommonExpenseEstimationPerPerson, Expense},
};
use ynab::types::ScheduledTransactionDetail;

/// Proportionally split common expenses
pub fn build_budget_summary(
    budget_details: &BudgetDetails,
    scheduled_transactions: &[ScheduledTransactionDetail],
    person_salary_settings: &[PersonSalarySettings],
) -> anyhow::Result<Vec<CommonExpenseEstimationPerPerson>> {
    let mut output: Vec<CommonExpenseEstimationPerPerson> = vec![];
    let salary_per_person =
        utils::get_salary_per_person_per_month(scheduled_transactions, person_salary_settings);

    let names_from_person_input: Vec<String> =
        salary_per_person.iter().map(|p| p.name.clone()).collect();

    output.extend(salary_per_person);

    let mut total_output = CommonExpenseEstimationPerPerson {
        name: String::from("Total"),
        salary: output.iter().map(|o| o.salary).sum(),
        salary_per_month: output.iter().map(|o| o.salary_per_month).sum(),
        ..Default::default()
    };

    let mut individual_expenses_per_person: HashMap<&str, Vec<&Expense>> = HashMap::new();

    total_output.common_expenses = budget_details
        .expenses
        .iter()
        .filter(|&e| !e.is_external)
        .filter(|&e| {
            let mut keep_in_it = true;

            names_from_person_input.iter().for_each(|name| {
                if e.name.contains(name) && e.projected_amount > 0 {
                    keep_in_it = false;
                    individual_expenses_per_person
                        .entry(name)
                        .or_insert_with(Vec::new)
                        .push(e);
                }
            });

            keep_in_it
        })
        .map(|e| e.projected_amount)
        .sum();

    for o in &mut output {
        o.proportion = o.salary_per_month as f64 / total_output.salary_per_month as f64;
        // The divide and multiply by 1000 is needed to convert from and to YNAB's milliunits format.
        o.common_expenses =
            ((o.proportion * (total_output.common_expenses as f64) / 1000_f64) * 1000_f64) as i64;
        o.individual_expenses = match individual_expenses_per_person.get(o.name.as_str()) {
            None => 0,
            Some(ie) => ie.iter().map(|&e| e.projected_amount).sum(),
        };
        o.left_over = o.salary_per_month - o.common_expenses - o.individual_expenses;
    }

    total_output.proportion = output.iter().map(|o| o.proportion).sum();
    total_output.individual_expenses = output.iter().map(|o| o.individual_expenses).sum();
    total_output.left_over = output.iter().map(|o| o.left_over).sum();
    output.push(total_output);
    Ok(output)
}
