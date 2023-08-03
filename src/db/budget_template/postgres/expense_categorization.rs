use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::budget_template::ExpenseCategorizationRepo, error::DatamizeResult,
    models::budget_template::ExpenseCategorization,
};

#[derive(Debug, Clone)]
pub struct PostgresExpenseCategorizationRepo {
    pub db_conn_pool: PgPool,
}

#[async_trait]
impl ExpenseCategorizationRepo for PostgresExpenseCategorizationRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DatamizeResult<Vec<ExpenseCategorization>> {
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
        .fetch_all(&self.db_conn_pool)
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

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: Uuid) -> DatamizeResult<ExpenseCategorization> {
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
        .fetch_one(&self.db_conn_pool)
        .await?;

        Ok(ExpenseCategorization {
            id: row.id,
            name: row.name,
            expense_type: row.r#type.parse().unwrap(),
            sub_expense_type: row.sub_type.parse().unwrap(),
        })
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(
        &self,
        expenses_categorization: &[ExpenseCategorization],
    ) -> DatamizeResult<()> {
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
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, expense_categorization: &ExpenseCategorization) -> DatamizeResult<()> {
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
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}
