use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{ynab::YnabPayeeRepo, DbResult},
    Uuid,
};
use sqlx::SqlitePool;
use ynab::Payee;

#[derive(Debug, Clone)]
pub struct SqliteYnabPayeeRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteYnabPayeeRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl YnabPayeeRepo for SqliteYnabPayeeRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<Payee>> {
        sqlx::query_as!(
            Payee,
            r#"
            SELECT 
                id as "id: Uuid",
                name,
                transfer_account_id as "transfer_account_id?: Uuid",
                deleted
            FROM payees
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

pub async fn sabotage_payees_table(pool: &SqlitePool) -> DbResult<()> {
    sqlx::query!("ALTER TABLE payees DROP COLUMN name;",)
        .execute(pool)
        .await?;

    Ok(())
}
