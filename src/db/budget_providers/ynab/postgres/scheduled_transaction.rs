use async_trait::async_trait;
use sqlx::PgPool;
use ynab::ScheduledTransactionDetail;

use crate::{db::budget_providers::ynab::YnabScheduledTransactionRepo, error::DatamizeResult};

#[derive(Debug, Clone)]
pub struct PostgresYnabScheduledTransactionRepo {
    pub db_conn_pool: PgPool,
}

#[async_trait]
impl YnabScheduledTransactionRepo for PostgresYnabScheduledTransactionRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DatamizeResult<Vec<ScheduledTransactionDetail>> {
        let scheduled_transactions_rows = sqlx::query!(
            r#"
            SELECT 
                id,
                date_first,
                date_next,
                frequency,
                amount,
                memo,
                flag_color,
                account_id,
                payee_id,
                category_id,
                transfer_account_id,
                deleted,
                account_name,
                payee_name,
                category_name,
                subtransactions
            FROM scheduled_transactions
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut saved_scheduled_transactions: Vec<ScheduledTransactionDetail> = vec![];

        for str in scheduled_transactions_rows {
            saved_scheduled_transactions.push(ScheduledTransactionDetail {
                id: str.id,
                date_first: str.date_first,
                date_next: str.date_next,
                frequency: str.frequency.map(|v| v.parse().unwrap()),
                amount: str.amount,
                memo: str.memo,
                flag_color: str.flag_color,
                account_id: str.account_id,
                payee_id: str.payee_id,
                category_id: str.category_id,
                transfer_account_id: str.transfer_account_id,
                deleted: str.deleted,
                account_name: str.account_name,
                payee_name: str.payee_name,
                category_name: str.category_name,
                subtransactions: serde_json::from_value(str.subtransactions).unwrap(),
            })
        }
        let saved_scheduled_transactions = saved_scheduled_transactions;
        Ok(saved_scheduled_transactions)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(
        &self,
        scheduled_transactions: &[ScheduledTransactionDetail],
    ) -> DatamizeResult<()> {
        for st in scheduled_transactions {
            if st.deleted {
                sqlx::query!(
                    r#"
                    DELETE FROM scheduled_transactions
                    WHERE id = $1
                    "#,
                    st.id
                )
                .execute(&self.db_conn_pool)
                .await?;
            } else {
                sqlx::query!(
                    r#"
                    INSERT INTO scheduled_transactions (id, date_first, date_next, frequency, amount, memo, flag_color, account_id, payee_id, category_id, transfer_account_id, deleted, account_name, payee_name, category_name, subtransactions)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                    ON CONFLICT (id) DO UPDATE
                    SET date_first = EXCLUDED.date_first,
                    date_next = EXCLUDED.date_next,
                    frequency = EXCLUDED.frequency,
                    amount = EXCLUDED.amount,
                    memo = EXCLUDED.memo,
                    flag_color = EXCLUDED.flag_color,
                    account_id = EXCLUDED.account_id,
                    payee_id = EXCLUDED.payee_id,
                    category_id = EXCLUDED.category_id,
                    transfer_account_id = EXCLUDED.transfer_account_id,
                    deleted = EXCLUDED.deleted,
                    account_name = EXCLUDED.account_name,
                    payee_name = EXCLUDED.payee_name,
                    category_name = EXCLUDED.category_name,
                    subtransactions = EXCLUDED.subtransactions;
                    "#,
                    st.id,
                    st.date_first,
                    st.date_next,
                    st.frequency.as_ref().map(|f| f.to_string()),
                    st.amount,
                    st.memo,
                    st.flag_color,
                    st.account_id,
                    st.payee_id,
                    st.category_id,
                    st.transfer_account_id,
                    st.deleted,
                    st.account_name,
                    st.payee_name,
                    st.category_name,
                    serde_json::to_value(&st.subtransactions).unwrap()
                ).execute(&self.db_conn_pool).await?;
            }
        }
        Ok(())
    }
}
