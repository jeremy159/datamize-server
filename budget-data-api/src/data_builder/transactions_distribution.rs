use std::collections::HashMap;

use super::types::{
    ExtendedScheduledTransactionDetail, ExtendedSubTransaction, ScheduledTransactionsDistribution,
    ScheduledTransactionsDistributionMap,
};
use super::utils;
use anyhow::Result;
use ynab::types::ScheduledTransactionDetail;

/// Gives the scheduled transactions distribution over a month.
pub async fn scheduled_transactions(
    scheduled_transactions: &[ScheduledTransactionDetail],
    category_id_to_name_map: &HashMap<uuid::Uuid, String>,
) -> Result<ScheduledTransactionsDistribution> {
    let mut output: ScheduledTransactionsDistributionMap = HashMap::new();

    let mut extended_scheduled_transactions: Vec<ExtendedScheduledTransactionDetail> = vec![];
    let mut repeated_sts: Vec<ExtendedScheduledTransactionDetail> = vec![];

    extended_scheduled_transactions.extend(
        scheduled_transactions
            .iter()
            .filter(|st| !st.deleted)
            .filter(|st| utils::is_transaction_in_next_30_days(&st.date_next))
            .map(|st| {
                let mut extended_st: ExtendedScheduledTransactionDetail = st.clone().into();
                repeated_sts.extend(
                    utils::find_repeatable_transactions(st)
                        .into_iter()
                        .map(|rep_st| rep_st.into()),
                );

                if !st.subtransactions.is_empty() {
                    extended_st.subtransactions.extend(
                        st.subtransactions
                            .iter()
                            .filter(|&sub_t| !sub_t.deleted)
                            .map(|sub_t| {
                                let category_name = match &sub_t.category_id {
                                    Some(id) => match category_id_to_name_map.get(id) {
                                        Some(name) => Some(name.clone()),
                                        None => st.category_name.clone(),
                                    },
                                    None => st.category_name.clone(),
                                };
                                ExtendedSubTransaction::from_sub_trans_and_trans(
                                    sub_t.clone(),
                                    st.clone(),
                                )
                                .with_category_name(category_name)
                            })
                            .collect::<Vec<_>>(),
                    );
                }

                extended_st
            }),
    );

    extended_scheduled_transactions.extend(repeated_sts);

    for st in &extended_scheduled_transactions {
        if !st.subtransactions.is_empty() {
            st.subtransactions.iter().for_each(|sub_st| {
                output
                    .entry(sub_st.date_next.to_string())
                    .and_modify(|v| v.push(sub_st.clone().into()))
                    .or_insert_with(|| vec![sub_st.clone().into()]);
            });
        } else {
            output
                .entry(st.scheduled_transaction.date_next.to_string())
                .and_modify(|v| v.push(st.clone()))
                .or_insert_with(|| vec![st.clone()]);
        }
    }

    Ok(ScheduledTransactionsDistribution { map: output })
}
