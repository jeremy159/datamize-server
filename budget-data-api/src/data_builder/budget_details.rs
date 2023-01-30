use std::collections::HashMap;

use super::types::{BudgetDetails, Expanse};
use super::utils;
use crate::config::types::{BugdetCalculationDataConfig, ExpanseType, SubExpanseType};
use anyhow::Result;
use uuid::Uuid;
use ynab::types::{Category, ScheduledTransactionDetail};

/// Gives bugdet details, spliting expanses by their type and sub-type.
pub fn budget_details(
    categories: &[Category],
    scheduled_transactions: &[ScheduledTransactionDetail],
    config: &BugdetCalculationDataConfig,
) -> Result<BudgetDetails> {
    let scheduled_transactions_map =
        build_category_to_scheduled_transaction_map(scheduled_transactions);

    let mut output: BudgetDetails = BudgetDetails::default();
    output.expanses.extend(
        categories
            .iter()
            .filter(|c| !c.hidden)
            .map(|c| {
                let projected_amount: i64 = get_projected_amount(c, &scheduled_transactions_map);
                let current_amount: i64 = get_current_amount(c, &scheduled_transactions_map);

                let (expanse_type, sub_expanse_type) =
                    get_expanse_categorization(&c.category_group_id, config);

                Expanse::new(
                    c.id,
                    c.name.clone(),
                    expanse_type,
                    sub_expanse_type,
                    projected_amount,
                    current_amount,
                )
            })
            .collect::<Vec<_>>(),
    );

    output
        .expanses
        .sort_by(|first, second| first.sub_expanse_type.cmp(&second.sub_expanse_type));

    // Filter out undefined
    output
        .expanses
        .retain(|e| e.expanse_type != ExpanseType::Undefined);

    output
        .expanses
        .extend(config.external_expanses.iter().map(|ee| ee.clone().into()));

    // Compute monthly income total
    let salary_per_person = utils::get_salary_per_person_per_month(scheduled_transactions);
    output.global.monthly_income = salary_per_person.iter().map(|sp| sp.salary_per_month).sum();
    let health_insurance_amount = match output
        .expanses
        .iter()
        .find(|c| c.name.contains("Assurance Santé"))
    {
        Some(c) => c.projected_amount,
        None => 0,
    };
    let retirement_savings_total = output
        .expanses
        .iter()
        .filter(|e| e.expanse_type == ExpanseType::RetirementSaving)
        .map(|c| c.projected_amount)
        .sum::<i64>();

    output.global.total_monthly_income =
        output.global.monthly_income + health_insurance_amount + retirement_savings_total;

    // Compute proportion for each expanse
    output.expanses.iter_mut().for_each(|e| {
        e.projected_proportion =
            e.projected_amount as f64 / output.global.total_monthly_income as f64;
        e.current_proportion = e.current_amount as f64 / output.global.total_monthly_income as f64;
    });
    Ok(output)
}

fn build_category_to_scheduled_transaction_map(
    scheduled_transactions: &[ScheduledTransactionDetail],
) -> HashMap<Uuid, Vec<ScheduledTransactionDetail>> {
    let mut hash_map: HashMap<Uuid, Vec<ScheduledTransactionDetail>> =
        HashMap::with_capacity(scheduled_transactions.len());

    let mut scheduled_transactions_filtered: Vec<&ScheduledTransactionDetail> = vec![];
    let mut repeated_sts: Vec<ScheduledTransactionDetail> = vec![];

    scheduled_transactions_filtered.extend(
        scheduled_transactions
            .iter()
            .filter(|st| st.category_id.is_some())
            .filter(|st| !st.deleted)
            .filter(|st| utils::is_transaction_in_next_30_days(&st.date_next))
            .map(|st| {
                repeated_sts.extend(utils::find_repeatable_transactions(st));
                st
            }),
    );

    scheduled_transactions_filtered.extend(&repeated_sts);

    for st in scheduled_transactions_filtered {
        let non_deleted_sub_st: Vec<_> = st
            .subtransactions
            .iter()
            .filter(|sub_st| !sub_st.deleted)
            .collect();

        if !non_deleted_sub_st.is_empty() {
            for sub_st in &non_deleted_sub_st {
                let transformed_st: ScheduledTransactionDetail =
                    st.clone().from_subtransaction(sub_st);

                let category_id = sub_st.category_id.unwrap();
                hash_map
                    .entry(category_id)
                    .and_modify(|v| v.push(transformed_st.clone()))
                    .or_insert_with(|| vec![transformed_st]);
            }
        } else {
            let category_id = st.category_id.unwrap();
            hash_map
                .entry(category_id)
                .and_modify(|v| v.push(st.clone()))
                .or_insert_with(|| vec![st.clone()]);
        }
    }

    hash_map
}

fn get_projected_amount(
    category: &Category,
    scheduled_transactions_map: &HashMap<Uuid, Vec<ScheduledTransactionDetail>>,
) -> i64 {
    let projected_amount = match category.goal_under_funded {
        Some(0) => {
            if let Some(percent) = category.goal_percentage_complete {
                match category.goal_target_month {
                    Some(_) => {
                        if percent == 100 {
                            category.budgeted
                        } else {
                            match category.goal_months_to_budget {
                                Some(months_remaining) => match category.goal_overall_left {
                                    Some(overall_left) => {
                                        (overall_left + category.budgeted) / months_remaining
                                    }
                                    None => category.goal_target,
                                },
                                None => category.goal_target,
                            }
                        }
                    }
                    None => category.goal_target,
                }
            } else {
                panic!("Should not be possible to have a 'goal_under_funded' but no 'goal_percentage_complete'.");
            }
        }
        Some(i) => i + category.budgeted,
        None => 0,
    };

    projected_amount
        + match scheduled_transactions_map.get(&category.id) {
            // Check with scheduled_transactions
            Some(t) => -t.iter().map(|v| v.amount).sum::<i64>(),
            None => 0,
        }
}

fn get_current_amount(
    category: &Category,
    scheduled_transactions_map: &HashMap<Uuid, Vec<ScheduledTransactionDetail>>,
) -> i64 {
    let current_amount = match category.goal_under_funded {
        Some(0) => category.budgeted,
        Some(i) => i + category.budgeted,
        None => 0,
    };

    current_amount
        + match scheduled_transactions_map.get(&category.id) {
            // Check with scheduled_transactions
            Some(t) => -t.iter().map(|v| v.amount).sum::<i64>(),
            None => 0,
        }
}

fn get_expanse_categorization(
    category_group_id: &Uuid,
    config: &BugdetCalculationDataConfig,
) -> (ExpanseType, SubExpanseType) {
    if config
        .fixed_expanses
        .housing_ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (ExpanseType::Fixed, SubExpanseType::Housing)
    } else if config
        .fixed_expanses
        .transport_ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (ExpanseType::Fixed, SubExpanseType::Transport)
    } else if config
        .fixed_expanses
        .other_ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (ExpanseType::Fixed, SubExpanseType::OtherFixed)
    } else if config
        .variable_expanses
        .subscription_ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (ExpanseType::Variable, SubExpanseType::Subscription)
    } else if config
        .variable_expanses
        .other_ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (ExpanseType::Variable, SubExpanseType::OtherVariable)
    } else if config
        .short_term_savings
        .ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (
            ExpanseType::ShortTermSaving,
            SubExpanseType::ShortTermSaving,
        )
    } else if config
        .long_term_savings
        .ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (ExpanseType::LongTermSaving, SubExpanseType::LongTermSaving)
    } else if config
        .retirement_savings
        .ids
        .iter()
        .any(|v| v == category_group_id)
    {
        (
            ExpanseType::RetirementSaving,
            SubExpanseType::RetirementSaving,
        )
    } else {
        (ExpanseType::Undefined, SubExpanseType::Undefined)
    }
}
