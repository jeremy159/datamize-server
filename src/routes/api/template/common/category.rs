use anyhow::Context;
use ynab::types::Category;

use crate::db::budget_providers::ynab::{
    get_categories, get_categories_delta, save_categories, set_categories_detla,
};

pub async fn get_latest_categories(
    db_conn_pool: &sqlx::PgPool,
    redis_conn: &mut redis::Connection,
    ynab_client: &ynab::Client,
) -> anyhow::Result<Vec<Category>> {
    let saved_categories_delta = get_categories_delta(redis_conn);

    let category_groups_with_categories_delta = ynab_client
        .get_categories_delta(saved_categories_delta)
        .await
        .context("failed to get categories from ynab's API")?;

    let categories = category_groups_with_categories_delta
        .category_groups
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect::<Vec<_>>();

    save_categories(db_conn_pool, &categories)
        .await
        .context("failed to save categories in database")?;

    set_categories_detla(
        redis_conn,
        category_groups_with_categories_delta.server_knowledge,
    )
    .context("failed to save last known server knowledge of categories in redis")?;

    get_categories(db_conn_pool)
        .await
        .context("failed to get categories from database")
}
