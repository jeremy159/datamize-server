use sqlx::PgPool;
use uuid::Uuid;

use crate::models::budget_template::ExpenseCategorization;

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_expenses_categorization(
    db_conn_pool: &PgPool,
) -> Result<Vec<ExpenseCategorization>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT
            id,
            name,
            type,
            sub_type
        FROM expenses_categorization
        "#
    )
    .fetch_all(db_conn_pool)
    .await?
    .into_iter()
    .map(|row| ExpenseCategorization {
        id: row.id,
        name: row.name,
        expense_type: row.r#type.parse().unwrap(),
        sub_expense_type: row.sub_type.parse().unwrap(),
    })
    .collect())
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_expense_categorization(
    db_conn_pool: &PgPool,
    id: Uuid,
) -> Result<ExpenseCategorization, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            name,
            type,
            sub_type
        FROM expenses_categorization
        WHERE id = $1;
        "#,
        id,
    )
    .fetch_one(db_conn_pool)
    .await?;

    Ok(ExpenseCategorization {
        id: row.id,
        name: row.name,
        expense_type: row.r#type.parse().unwrap(),
        sub_expense_type: row.sub_type.parse().unwrap(),
    })
}

#[tracing::instrument(skip_all)]
pub async fn update_all_expenses_categorization(
    db_conn_pool: &PgPool,
    expenses_categorization: &[ExpenseCategorization],
) -> Result<(), sqlx::Error> {
    for expense_categorization in expenses_categorization {
        sqlx::query!(
            r#"
        INSERT INTO expenses_categorization (id, name, type, sub_type)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
        type = EXCLUDED.type,
        sub_type = EXCLUDED.sub_type;
        "#,
            expense_categorization.id,
            expense_categorization.name,
            expense_categorization.expense_type.to_string(),
            expense_categorization.sub_expense_type.to_string(),
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn update_expense_categorization(
    db_conn_pool: &PgPool,
    expense_categorization: &ExpenseCategorization,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO expenses_categorization (id, name, type, sub_type)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
        type = EXCLUDED.type,
        sub_type = EXCLUDED.sub_type;
        "#,
        expense_categorization.id,
        expense_categorization.name,
        expense_categorization.expense_type.to_string(),
        expense_categorization.sub_expense_type.to_string(),
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}
