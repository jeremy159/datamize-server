use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::budget_template::ExternalExpenseRepo,
    error::DatamizeResult,
    models::budget_template::{ExpenseType, ExternalExpense, SubExpenseType},
};

#[derive(Debug, Clone)]
pub struct PostgresExternalExpenseRepo {
    pub db_conn_pool: PgPool,
}

#[async_trait]
impl ExternalExpenseRepo for PostgresExternalExpenseRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DatamizeResult<Vec<ExternalExpense>> {
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
        .fetch_all(&self.db_conn_pool)
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

    #[tracing::instrument(skip(self))]
    async fn get(&self, expense_id: Uuid) -> DatamizeResult<ExternalExpense> {
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
            expense_id,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        Ok(ExternalExpense {
            id: row.id,
            name: row.name,
            projected_amount: row.projected_amount,
            expense_type: row.r#type.parse().unwrap(),
            sub_expense_type: row.sub_type.parse().unwrap(),
        })
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_name(&self, name: &str) -> DatamizeResult<ExternalExpense> {
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
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, expense: &ExternalExpense) -> DatamizeResult<()> {
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
            expense.id,
            expense.name,
            expense.expense_type.to_string(),
            expense.sub_expense_type.to_string(),
            expense.projected_amount,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, expense_id: Uuid) -> DatamizeResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM external_expenses
                WHERE id = $1
            "#,
            expense_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}
