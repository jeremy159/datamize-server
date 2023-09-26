use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use ynab::{BaseTransactionDetail, TransactionDetail};

use crate::{db::budget_providers::ynab::YnabTransactionRepo, error::DatamizeResult};

#[derive(Debug, Clone)]
pub struct PostgresYnabTransactionRepo {
    pub db_conn_pool: PgPool,
}

#[async_trait]
impl YnabTransactionRepo for PostgresYnabTransactionRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DatamizeResult<Vec<TransactionDetail>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT 
                id,
                date,
                amount,
                memo,
                cleared,
                approved,
                flag_color,
                account_id,
                payee_id,
                category_id,
                transfer_account_id,
                transfer_transaction_id,
                matched_transaction_id,
                import_id,
                deleted,
                account_name,
                payee_name,
                category_name,
                subtransactions
            FROM transactions
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| TransactionDetail {
                base: BaseTransactionDetail {
                    id: r.id,
                    date: r.date,
                    amount: r.amount,
                    memo: r.memo,
                    cleared: r.cleared.parse().unwrap(),
                    approved: r.approved,
                    flag_color: r.flag_color,
                    account_id: r.account_id,
                    payee_id: r.payee_id,
                    category_id: r.category_id,
                    transfer_account_id: r.transfer_account_id,
                    transfer_transaction_id: r.transfer_transaction_id,
                    matched_transaction_id: r.matched_transaction_id,
                    import_id: r.import_id,
                    deleted: r.deleted,
                    account_name: r.account_name,
                    payee_name: r.payee_name,
                    category_name: r.category_name,
                },
                subtransactions: serde_json::from_value(r.subtransactions).unwrap(),
            })
            .collect())
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, transactions: &[TransactionDetail]) -> DatamizeResult<()> {
        for t in transactions {
            sqlx::query!(
                    r#"
                    INSERT INTO transactions (id, date, amount, memo, cleared, approved, flag_color, account_id, payee_id, category_id, transfer_account_id, transfer_transaction_id, matched_transaction_id, import_id, deleted, account_name, payee_name, category_name, subtransactions)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                    ON CONFLICT (id) DO UPDATE SET
                    date = EXCLUDED.date,
                    amount = EXCLUDED.amount,
                    memo = EXCLUDED.memo,
                    cleared = EXCLUDED.cleared,
                    approved = EXCLUDED.approved,
                    flag_color = EXCLUDED.flag_color,
                    account_id = EXCLUDED.account_id,
                    payee_id = EXCLUDED.payee_id,
                    category_id = EXCLUDED.category_id,
                    transfer_account_id = EXCLUDED.transfer_account_id,
                    transfer_transaction_id = EXCLUDED.transfer_transaction_id,
                    matched_transaction_id = EXCLUDED.matched_transaction_id,
                    import_id = EXCLUDED.import_id,
                    deleted = EXCLUDED.deleted,
                    account_name = EXCLUDED.account_name,
                    payee_name = EXCLUDED.payee_name,
                    category_name = EXCLUDED.category_name,
                    subtransactions = EXCLUDED.subtransactions;
                    "#,
                    t.base.id,
                    t.base.date,
                    t.base.amount,
                    t.base.memo,
                    t.base.cleared.to_string(),
                    t.base.approved,
                    t.base.flag_color,
                    t.base.account_id,
                    t.base.payee_id,
                    t.base.category_id,
                    t.base.transfer_account_id,
                    t.base.transfer_transaction_id,
                    t.base.matched_transaction_id,
                    t.base.import_id,
                    t.base.deleted,
                    t.base.account_name,
                    t.base.payee_name,
                    t.base.category_name,
                    serde_json::to_value(&t.subtransactions).unwrap()
                ).execute(&self.db_conn_pool).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_with_payee_id(
        &self,
        payee_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT 
                id,
                date,
                amount,
                memo,
                cleared,
                approved,
                flag_color,
                account_id,
                payee_id,
                category_id,
                transfer_account_id,
                transfer_transaction_id,
                matched_transaction_id,
                import_id,
                deleted,
                account_name,
                payee_name,
                category_name,
                subtransactions
            FROM transactions
            WHERE payee_id = $1
            "#,
            payee_id
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| TransactionDetail {
                base: BaseTransactionDetail {
                    id: r.id,
                    date: r.date,
                    amount: r.amount,
                    memo: r.memo,
                    cleared: r.cleared.parse().unwrap(),
                    approved: r.approved,
                    flag_color: r.flag_color,
                    account_id: r.account_id,
                    payee_id: r.payee_id,
                    category_id: r.category_id,
                    transfer_account_id: r.transfer_account_id,
                    transfer_transaction_id: r.transfer_transaction_id,
                    matched_transaction_id: r.matched_transaction_id,
                    import_id: r.import_id,
                    deleted: r.deleted,
                    account_name: r.account_name,
                    payee_name: r.payee_name,
                    category_name: r.category_name,
                },
                subtransactions: serde_json::from_value(r.subtransactions).unwrap(),
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_with_category_id(
        &self,
        category_id: Uuid,
    ) -> DatamizeResult<Vec<TransactionDetail>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT 
                id,
                date,
                amount,
                memo,
                cleared,
                approved,
                flag_color,
                account_id,
                payee_id,
                category_id,
                transfer_account_id,
                transfer_transaction_id,
                matched_transaction_id,
                import_id,
                deleted,
                account_name,
                payee_name,
                category_name,
                subtransactions
            FROM transactions
            WHERE category_id = $1
            "#,
            category_id
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| TransactionDetail {
                base: BaseTransactionDetail {
                    id: r.id,
                    date: r.date,
                    amount: r.amount,
                    memo: r.memo,
                    cleared: r.cleared.parse().unwrap(),
                    approved: r.approved,
                    flag_color: r.flag_color,
                    account_id: r.account_id,
                    payee_id: r.payee_id,
                    category_id: r.category_id,
                    transfer_account_id: r.transfer_account_id,
                    transfer_transaction_id: r.transfer_transaction_id,
                    matched_transaction_id: r.matched_transaction_id,
                    import_id: r.import_id,
                    deleted: r.deleted,
                    account_name: r.account_name,
                    payee_name: r.payee_name,
                    category_name: r.category_name,
                },
                subtransactions: serde_json::from_value(r.subtransactions).unwrap(),
            })
            .collect())
    }
}
