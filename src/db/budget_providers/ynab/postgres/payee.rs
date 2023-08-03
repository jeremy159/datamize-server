use async_trait::async_trait;
use sqlx::PgPool;
use ynab::Payee;

use crate::{db::budget_providers::ynab::YnabPayeeRepo, error::DatamizeResult};

#[derive(Debug, Clone)]
pub struct PostgresYnabPayeeRepo {
    pub db_conn_pool: PgPool,
}

#[async_trait]
impl YnabPayeeRepo for PostgresYnabPayeeRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DatamizeResult<Vec<Payee>> {
        sqlx::query_as!(
            Payee,
            r#"
            SELECT 
                id,
                name,
                transfer_account_id,
                deleted
            FROM payees
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, payees: &[Payee]) -> DatamizeResult<()> {
        for p in payees {
            sqlx::query!(
                r#"
                    INSERT INTO payees (id, name, transfer_account_id, deleted)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (id) DO UPDATE SET
                    name = EXCLUDED.name,
                    transfer_account_id = EXCLUDED.transfer_account_id,
                    deleted = EXCLUDED.deleted;
                    "#,
                p.id,
                p.name,
                p.transfer_account_id,
                p.deleted
            )
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }
}
