use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{BudgeterConfigRepo, DbResult},
    BudgeterConfig, Uuid,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteBudgeterConfigRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteBudgeterConfigRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl BudgeterConfigRepo for SqliteBudgeterConfigRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<BudgeterConfig>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                id as "id: Uuid",
                name,
                payee_ids
            FROM budgeters_config
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| {
                let payee_ids_rec: IdsRecord = serde_json::from_str(&r.payee_ids).unwrap();
                BudgeterConfig {
                    id: r.id,
                    name: r.name,
                    payee_ids: payee_ids_rec.ids,
                }
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, budgeter_id: Uuid) -> DbResult<BudgeterConfig> {
        let db_row = sqlx::query!(
            r#"
            SELECT
                id as "id: Uuid",
                name,
                payee_ids
            FROM budgeters_config
            WHERE id = $1;
            "#,
            budgeter_id,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        let payee_ids_rec: IdsRecord = serde_json::from_str(&db_row.payee_ids).unwrap();

        Ok(BudgeterConfig {
            id: db_row.id,
            name: db_row.name,
            payee_ids: payee_ids_rec.ids,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_name(&self, name: &str) -> DbResult<BudgeterConfig> {
        let db_row = sqlx::query!(
            r#"
            SELECT
                id as "id: Uuid",
                name,
                payee_ids
            FROM budgeters_config
            WHERE name = $1;
            "#,
            name,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        let payee_ids_rec: IdsRecord = serde_json::from_str(&db_row.payee_ids).unwrap();

        Ok(BudgeterConfig {
            id: db_row.id,
            name: db_row.name,
            payee_ids: payee_ids_rec.ids,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, budgeter: &BudgeterConfig) -> DbResult<()> {
        let payee_ids = serde_json::to_string(&IdsRecord {
            ids: budgeter.payee_ids.clone(),
        })
        .unwrap();

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
            payee_ids,
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

#[derive(Debug, Serialize, Deserialize)]
struct IdsRecord {
    ids: Vec<Uuid>,
}

pub async fn sabotage_budgeters_config_table(pool: &SqlitePool) -> DbResult<()> {
    sqlx::query!("ALTER TABLE budgeters_config DROP COLUMN name;",)
        .execute(pool)
        .await?;

    Ok(())
}
