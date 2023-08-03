use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use ynab::{Category, GoalType};

use crate::{db::budget_providers::ynab::YnabCategoryRepo, error::DatamizeResult};

#[derive(Debug, Clone)]
pub struct PostgresYnabCategoryRepo {
    pub db_conn_pool: PgPool,
}

#[async_trait]
impl YnabCategoryRepo for PostgresYnabCategoryRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DatamizeResult<Vec<Category>> {
        sqlx::query_as!(
            Category,
            r#"
            SELECT 
                id,
                category_group_id,
                category_group_name,
                name,
                hidden,
                original_category_group_id,
                note,
                budgeted,
                activity,
                balance,
                goal_type AS "goal_type?: GoalType",
                goal_creation_month,
                goal_target,
                goal_target_month,
                goal_percentage_complete,
                goal_months_to_budget,
                goal_under_funded,
                goal_overall_funded,
                goal_overall_left,
                deleted,
                goal_day,
                goal_cadence,
                goal_cadence_frequency
            FROM categories
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, category_id: Uuid) -> DatamizeResult<Category> {
        sqlx::query_as!(
            Category,
            r#"
            SELECT 
                id,
                category_group_id,
                category_group_name,
                name,
                hidden,
                original_category_group_id,
                note,
                budgeted,
                activity,
                balance,
                goal_type AS "goal_type?: GoalType",
                goal_creation_month,
                goal_target,
                goal_target_month,
                goal_percentage_complete,
                goal_months_to_budget,
                goal_under_funded,
                goal_overall_funded,
                goal_overall_left,
                deleted,
                goal_day,
                goal_cadence,
                goal_cadence_frequency
            FROM categories
            WHERE id = $1
            "#,
            category_id,
        )
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, categories: &[Category]) -> DatamizeResult<()> {
        for c in categories {
            sqlx::query!(
                    r#"
                    INSERT INTO categories (id, category_group_id, category_group_name, name, hidden, original_category_group_id, note, budgeted, activity, balance, goal_type, goal_creation_month, goal_target, goal_target_month, goal_percentage_complete, goal_months_to_budget, goal_under_funded, goal_overall_funded, goal_overall_left, deleted, goal_day, goal_cadence, goal_cadence_frequency)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
                    ON CONFLICT (id) DO UPDATE SET
                    category_group_id = EXCLUDED.category_group_id,
                    category_group_name = EXCLUDED.category_group_name,
                    name = EXCLUDED.name,
                    hidden = EXCLUDED.hidden,
                    original_category_group_id = EXCLUDED.original_category_group_id,
                    note = EXCLUDED.note,
                    budgeted = EXCLUDED.budgeted,
                    activity = EXCLUDED.activity,
                    balance = EXCLUDED.balance,
                    goal_type = EXCLUDED.goal_type,
                    goal_creation_month = EXCLUDED.goal_creation_month,
                    goal_target = EXCLUDED.goal_target,
                    goal_target_month = EXCLUDED.goal_target_month,
                    goal_percentage_complete = EXCLUDED.goal_percentage_complete,
                    goal_months_to_budget = EXCLUDED.goal_months_to_budget,
                    goal_under_funded = EXCLUDED.goal_under_funded,
                    goal_overall_funded = EXCLUDED.goal_overall_funded,
                    goal_overall_left = EXCLUDED.goal_overall_left,
                    deleted = EXCLUDED.deleted,
                    goal_day = EXCLUDED.goal_day,
                    goal_cadence = EXCLUDED.goal_cadence,
                    goal_cadence_frequency = EXCLUDED.goal_cadence_frequency;
                    "#,
                    c.id,
                    c.category_group_id,
                    c.category_group_name,
                    c.name,
                    c.hidden,
                    c.original_category_group_id,
                    c.note,
                    c.budgeted,
                    c.activity,
                    c.balance,
                    c.goal_type.as_ref().map(|g| g.to_string()),
                    c.goal_creation_month,
                    c.goal_target,
                    c.goal_target_month,
                    c.goal_percentage_complete,
                    c.goal_months_to_budget,
                    c.goal_under_funded,
                    c.goal_overall_funded,
                    c.goal_overall_left,
                    c.deleted,
                    c.goal_day,
                    c.goal_cadence,
                    c.goal_cadence_frequency,
                ).execute(&self.db_conn_pool).await?;
        }

        Ok(())
    }
}
