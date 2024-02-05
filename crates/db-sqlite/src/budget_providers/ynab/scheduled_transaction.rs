use std::sync::Arc;

use chrono::NaiveDate;
use datamize_domain::{
    async_trait,
    db::{ynab::YnabScheduledTransactionRepo, DbResult},
    Uuid,
};
use sqlx::SqlitePool;
use ynab::{RecurFrequency, ScheduledTransactionDetail};

#[derive(Debug, Clone)]
pub struct SqliteYnabScheduledTransactionRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteYnabScheduledTransactionRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl YnabScheduledTransactionRepo for SqliteYnabScheduledTransactionRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<ScheduledTransactionDetail>> {
        let scheduled_transactions_rows = sqlx::query!(
            r#"
            SELECT 
                id as "id: Uuid",
                date_first as "date_first: NaiveDate",
                date_next as "date_next: NaiveDate",
                frequency as "frequency: RecurFrequency",
                amount,
                memo,
                flag_color,
                account_id as "account_id: Uuid",
                payee_id as "payee_id?: Uuid",
                category_id as "category_id?: Uuid",
                transfer_account_id as "transfer_account_id?: Uuid",
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
                frequency: str.frequency,
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
                subtransactions: serde_json::from_str(&str.subtransactions).unwrap(),
            })
        }
        let saved_scheduled_transactions = saved_scheduled_transactions;
        Ok(saved_scheduled_transactions)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(
        &self,
        scheduled_transactions: &[ScheduledTransactionDetail],
    ) -> DbResult<()> {
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
                let subtransactions = serde_json::to_string(&st.subtransactions).unwrap();

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
                    st.frequency,
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
                    subtransactions,
                ).execute(&self.db_conn_pool).await?;
            }
        }
        Ok(())
    }
}

pub async fn sabotage_scheduled_transactions_table(pool: &SqlitePool) -> DbResult<()> {
    sqlx::query!("ALTER TABLE scheduled_transactions DROP COLUMN date_next;",)
        .execute(pool)
        .await?;

    Ok(())
}
