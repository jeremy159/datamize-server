use sqlx::PgPool;
use uuid::Uuid;

use crate::models::budget_template::BudgeterConfig;

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_budgeters_config(
    db_conn_pool: &PgPool,
) -> Result<Vec<BudgeterConfig>, sqlx::Error> {
    sqlx::query_as!(
        BudgeterConfig,
        r#"
        SELECT
            *
        FROM budgeters_config
        "#
    )
    .fetch_all(db_conn_pool)
    .await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_budgeter_config(
    db_conn_pool: &PgPool,
    id: Uuid,
) -> Result<BudgeterConfig, sqlx::Error> {
    sqlx::query_as!(
        BudgeterConfig,
        r#"
        SELECT
            *
        FROM budgeters_config
        WHERE id = $1;
        "#,
        id,
    )
    .fetch_one(db_conn_pool)
    .await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_budgeter_config_by_name(
    db_conn_pool: &PgPool,
    name: &str,
) -> Result<BudgeterConfig, sqlx::Error> {
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
    .fetch_one(db_conn_pool)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn update_budgeter_config(
    db_conn_pool: &PgPool,
    budgeter_config: &BudgeterConfig,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO budgeters_config (id, name, payee_ids)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
        payee_ids = EXCLUDED.payee_ids;
        "#,
        budgeter_config.id,
        budgeter_config.name,
        budgeter_config.payee_ids.as_slice(),
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_budgeter_config(db_conn_pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM budgeters_config
            WHERE id = $1
        "#,
        id,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}
