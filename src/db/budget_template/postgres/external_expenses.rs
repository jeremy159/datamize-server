use sqlx::PgPool;
use uuid::Uuid;

use crate::models::budget_template::{ExpenseType, ExternalExpense, SubExpenseType};

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_external_expenses(
    db_conn_pool: &PgPool,
) -> Result<Vec<ExternalExpense>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        SELECT
            id,
            name,
            type,
            sub_type,
            projected_amount
        FROM external_expenses
        "#
    )
    .fetch_all(db_conn_pool)
    .await?
    .into_iter()
    .map(|row| ExternalExpense {
        id: row.id,
        name: row.name,
        projected_amount: row.projected_amount,
        expense_type: row.r#type.parse().unwrap(),
        sub_expense_type: row.sub_type.parse().unwrap(),
    })
    .collect())
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_external_expense(
    db_conn_pool: &PgPool,
    id: Uuid,
) -> Result<ExternalExpense, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            name,
            type,
            sub_type,
            projected_amount
        FROM external_expenses
        WHERE id = $1;
        "#,
        id,
    )
    .fetch_one(db_conn_pool)
    .await?;

    Ok(ExternalExpense {
        id: row.id,
        name: row.name,
        projected_amount: row.projected_amount,
        expense_type: row.r#type.parse().unwrap(),
        sub_expense_type: row.sub_type.parse().unwrap(),
    })
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_external_expense_by_name(
    db_conn_pool: &PgPool,
    name: &str,
) -> Result<ExternalExpense, sqlx::Error> {
    sqlx::query_as!(
        ExternalExpense,
        r#"
        SELECT
            id,
            name,
            type AS "expense_type: ExpenseType",
            sub_type AS "sub_expense_type: SubExpenseType",
            projected_amount
        FROM external_expenses
        WHERE name = $1;
        "#,
        name,
    )
    .fetch_one(db_conn_pool)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn update_external_expense(
    db_conn_pool: &PgPool,
    external_expense: &ExternalExpense,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO external_expenses (id, name, type, sub_type, projected_amount)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
        type = EXCLUDED.type,
        sub_type = EXCLUDED.sub_type,
        projected_amount = EXCLUDED.projected_amount;
        "#,
        external_expense.id,
        external_expense.name,
        external_expense.expense_type.to_string(),
        external_expense.sub_expense_type.to_string(),
        external_expense.projected_amount,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_external_expense(db_conn_pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM external_expenses
            WHERE id = $1
        "#,
        id,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}
