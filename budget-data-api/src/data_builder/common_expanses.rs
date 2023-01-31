use std::collections::HashMap;

use super::types::{BudgetDetails, CommonExpanseEstimationPerPerson, Expanse};
use super::utils;
use crate::Result;
use ynab::types::ScheduledTransactionDetail;

/// Proportionally split common expanses
pub fn common_expanses(
    budget_details: &BudgetDetails,
    scheduled_transactions: &[ScheduledTransactionDetail],
) -> Result<Vec<CommonExpanseEstimationPerPerson>> {
    let mut output: Vec<CommonExpanseEstimationPerPerson> = vec![];
    let salary_per_person = utils::get_salary_per_person_per_month(scheduled_transactions);

    output.extend(salary_per_person.clone());

    let mut total_output = CommonExpanseEstimationPerPerson {
        name: "Total".to_string(),
        salary: output.iter().map(|o| o.salary).sum(),
        salary_per_month: output.iter().map(|o| o.salary_per_month).sum(),
        ..Default::default()
    };

    let names_from_person_input: Vec<&str> =
        salary_per_person.iter().map(|p| p.name.as_str()).collect();
    let mut individual_expanses_per_person: HashMap<&str, Vec<&Expanse>> = HashMap::new();

    total_output.common_expanses = budget_details
        .expanses
        .iter()
        .filter(|&e| !e.is_external)
        .filter(|&e| {
            let mut keep_in_it = true;

            names_from_person_input.iter().for_each(|&n| {
                if e.name.contains(n) && e.projected_amount > 0 {
                    keep_in_it = false;
                    individual_expanses_per_person
                        .entry(n)
                        .and_modify(|ie| ie.push(e))
                        .or_insert_with(|| vec![e]);
                }
            });

            keep_in_it
        })
        .map(|e| e.projected_amount)
        .sum();

    for o in &mut output {
        o.proportion = o.salary_per_month as f64 / total_output.salary_per_month as f64;
        // The divide and multiply by 1000 is needed to convert from and to YNAB's milliunits format.
        o.common_expanses =
            ((o.proportion * (total_output.common_expanses as f64) / 1000_f64) * 1000_f64) as i64;
        o.individual_expanses = match individual_expanses_per_person.get(o.name.as_str()) {
            None => 0,
            Some(ie) => ie.iter().map(|&ec| ec.projected_amount).sum(),
        };
        o.left_over = o.salary_per_month - o.common_expanses - o.individual_expanses;
    }

    total_output.proportion = output.iter().map(|o| o.proportion).sum();
    total_output.individual_expanses = output.iter().map(|o| o.individual_expanses).sum();
    total_output.left_over = output.iter().map(|o| o.left_over).sum();
    output.push(total_output);
    Ok(output)
}
