use datamize_domain::{
    async_trait,
    db::{DbResult, ExpenseCategorizationRepo},
    ExpenseCategorization, ExpenseType, SubExpenseType, Uuid,
};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteExpenseCategorizationRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteExpenseCategorizationRepo {
    pub fn new_boxed(db_conn_pool: SqlitePool) -> Box<Self> {
        Box::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl ExpenseCategorizationRepo for SqliteExpenseCategorizationRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<ExpenseCategorization>> {
        sqlx::query_as!(
            ExpenseCategorization,
            r#"
            SELECT
                id as "id: Uuid",
                name,
                type as "expense_type: ExpenseType",
                sub_type as "sub_expense_type: SubExpenseType"
            FROM expenses_categorization
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, id: Uuid) -> DbResult<ExpenseCategorization> {
        sqlx::query_as!(
            ExpenseCategorization,
            r#"
            SELECT
                id as "id: Uuid",
                name,
                type as "expense_type: ExpenseType",
                sub_type as "sub_expense_type: SubExpenseType"
            FROM expenses_categorization
            WHERE id = $1;
            "#,
            id,
        )
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, expenses_categorization: &[ExpenseCategorization]) -> DbResult<()> {
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
                expense_categorization.expense_type,
                expense_categorization.sub_expense_type,
            )
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, expense_categorization: &ExpenseCategorization) -> DbResult<()> {
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
            expense_categorization.expense_type,
            expense_categorization.sub_expense_type,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}
