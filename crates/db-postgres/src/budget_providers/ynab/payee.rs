use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabPayeeRepo, DbResult},
};
use sqlx::PgPool;
use ynab::Payee;

#[derive(Debug, Clone)]
pub struct PostgresYnabPayeeRepo {
    pub db_conn_pool: PgPool,
}

impl PostgresYnabPayeeRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl YnabPayeeRepo for PostgresYnabPayeeRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<Payee>> {
        sqlx::query_as!(
            Payee,
            r#"
            SELECT 
                id,
                name,
                transfer_account_id,
                deleted
            FROM payees
            ORDER BY name
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, payees: &[Payee]) -> DbResult<()> {
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
