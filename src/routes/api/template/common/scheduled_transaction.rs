use std::collections::HashMap;

use anyhow::Context;
use ynab::types::ScheduledTransactionDetail;

use crate::{
    db::budget_providers::ynab::{
        get_scheduled_transactions, get_scheduled_transactions_delta, save_scheduled_transactions,
        set_scheduled_transactions_delta,
    },
    models::budget_template::{
        CategoryIdToNameMap, ExtendedScheduledTransactionDetail, ExtendedSubTransaction,
        ScheduledTransactionsDistribution, ScheduledTransactionsDistributionMap,
    },
};

use super::utils;

pub async fn get_latest_scheduled_transactions(
    db_conn_pool: &sqlx::PgPool,
    redis_conn: &mut redis::Connection,
    ynab_client: &ynab::Client,
) -> anyhow::Result<Vec<ScheduledTransactionDetail>> {
    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(redis_conn);

    let scheduled_transactions_delta = ynab_client
        .get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
        .await
        .context("failed to get scheduled transactions from ynab's API")?;

    save_scheduled_transactions(
        db_conn_pool,
        &scheduled_transactions_delta.scheduled_transactions,
    )
    .await
    .context("failed to save scheduled transactions in database")?;

    set_scheduled_transactions_delta(redis_conn, scheduled_transactions_delta.server_knowledge)
        .context("failed to save last known server knowledge of scheduled transactions in redis")?;

    get_scheduled_transactions(db_conn_pool)
        .await
        .context("failed to get scheduled transactions from database")
}

pub fn build_scheduled_transactions(
    scheduled_transactions: Vec<ScheduledTransactionDetail>,
    category_id_to_name_map: &CategoryIdToNameMap,
) -> anyhow::Result<ScheduledTransactionsDistribution> {
    let mut output: ScheduledTransactionsDistributionMap = HashMap::new();

    let mut extended_scheduled_transactions: Vec<ExtendedScheduledTransactionDetail> = vec![];
    let mut repeated_sts: Vec<ExtendedScheduledTransactionDetail> = vec![];

    extended_scheduled_transactions.extend(
        scheduled_transactions
            .into_iter()
            .filter(|st| !st.deleted)
            .filter(|st| utils::is_transaction_in_next_30_days(&st.date_next))
            .map(|st| {
                let mut extended_st: ExtendedScheduledTransactionDetail = st.clone().into();
                repeated_sts.extend(
                    utils::find_repeatable_transactions(&st)
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

    for st in extended_scheduled_transactions {
        if !st.subtransactions.is_empty() {
            st.subtransactions.into_iter().for_each(|sub_st| {
                let entry = output.entry(sub_st.date_next.to_string());
                entry.or_insert_with(Vec::new).push(sub_st.into());
            });
        } else {
            let entry = output.entry(st.scheduled_transaction.date_next.to_string());
            entry.or_insert_with(Vec::new).push(st);
        }
    }

    Ok(ScheduledTransactionsDistribution { map: output })
}
