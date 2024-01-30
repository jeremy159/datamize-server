use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{BudgeterConfigRepo, DbResult},
    BudgeterConfig, Uuid,
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PostgresBudgeterConfigRepo {
    pub db_conn_pool: PgPool,
}

impl PostgresBudgeterConfigRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl BudgeterConfigRepo for PostgresBudgeterConfigRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<BudgeterConfig>> {
        sqlx::query_as!(
            BudgeterConfig,
            r#"
            SELECT
                *
            FROM budgeters_config
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, budgeter_id: Uuid) -> DbResult<BudgeterConfig> {
        sqlx::query_as!(
            BudgeterConfig,
            r#"
            SELECT
                *
            FROM budgeters_config
            WHERE id = $1;
            "#,
            budgeter_id,
        )
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_name(&self, name: &str) -> DbResult<BudgeterConfig> {
        sqlx::query_as!(
            BudgeterConfig,
            r#"
            SELECT
                *
            FROM budgeters_config
            WHERE name = $1;
            "#,
            name,
        )
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, budgeter: &BudgeterConfig) -> DbResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO budgeters_config (id, name, payee_ids)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            payee_ids = EXCLUDED.payee_ids;
            "#,
            budgeter.id,
            budgeter.name,
            budgeter.payee_ids.as_slice(),
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, budgeter_id: Uuid) -> DbResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM budgeters_config
                WHERE id = $1
            "#,
            budgeter_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}
