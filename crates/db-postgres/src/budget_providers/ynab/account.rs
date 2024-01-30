use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabAccountRepo, DbResult},
};
use sqlx::PgPool;
use ynab::{Account, AccountType};

#[derive(Debug, Clone)]
pub struct PostgresYnabAccountRepo {
    pub db_conn_pool: PgPool,
}

impl PostgresYnabAccountRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl YnabAccountRepo for PostgresYnabAccountRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<Account>> {
        sqlx::query_as!(
            Account,
            r#"
            SELECT 
                id,
                name,
                type AS "account_type: AccountType",
                on_budget,
                closed,
                note,
                balance,
                cleared_balance,
                uncleared_balance,
                transfer_payee_id,
                direct_import_linked,
                direct_import_in_error,
                deleted
            FROM accounts
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, accounts: &[Account]) -> DbResult<()> {
        for a in accounts {
            sqlx::query!(
                    r#"
                    INSERT INTO accounts (id, name, type, on_budget, closed, note, balance, cleared_balance, uncleared_balance, transfer_payee_id, direct_import_linked, direct_import_in_error, deleted)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                    ON CONFLICT (id) DO UPDATE SET
                    name = EXCLUDED.name,
                    type = EXCLUDED.type,
                    on_budget = EXCLUDED.on_budget,
                    closed = EXCLUDED.closed,
                    note = EXCLUDED.note,
                    balance = EXCLUDED.balance,
                    cleared_balance = EXCLUDED.cleared_balance,
                    uncleared_balance = EXCLUDED.uncleared_balance,
                    transfer_payee_id = EXCLUDED.transfer_payee_id,
                    direct_import_linked = EXCLUDED.direct_import_linked,
                    direct_import_in_error = EXCLUDED.direct_import_in_error,
                    deleted = EXCLUDED.deleted;
                    "#,
                    a.id,
                    a.name,
                    a.account_type.to_string(),
                    a.on_budget,
                    a.closed,
                    a.note,
                    a.balance,
                    a.cleared_balance,
                    a.uncleared_balance,
                    a.transfer_payee_id,
                    a.direct_import_linked,
                    a.direct_import_in_error,
                    a.deleted
                ).execute(&self.db_conn_pool).await?;
        }

        Ok(())
    }
}
