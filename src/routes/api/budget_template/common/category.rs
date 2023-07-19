use anyhow::Context;
use chrono::{DateTime, Local};
use ynab::types::Category;

use crate::{
    db::{
        budget_providers::ynab::{
            get_categories, get_categories_delta, save_categories, set_categories_detla,
        },
        budget_template,
    },
    models::budget_template::{ExpenseCategorization, MonthTarget},
};

pub async fn get_latest_categories(
    db_conn_pool: &sqlx::PgPool,
    redis_conn: &mut redis::Connection,
    ynab_client: &ynab::Client,
) -> anyhow::Result<(Vec<Category>, Vec<ExpenseCategorization>)> {
    let saved_categories_delta = get_categories_delta(redis_conn);

    let category_groups_with_categories_delta = ynab_client
        .get_categories_delta(saved_categories_delta)
        .await
        .context("failed to get categories from ynab's API")?;

    let categories = category_groups_with_categories_delta
        .category_groups
        .clone() // TODO: Find better way of doing this...
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect::<Vec<_>>();

    let mut expenses_categorization: Vec<ExpenseCategorization> = vec![];

    for cg in category_groups_with_categories_delta
        .category_groups
        .into_iter()
        .filter(|cg| !cg.deleted && !cg.hidden)
    // TODO: Also filter special ynab category groups
    {
        let expense_categorization: ExpenseCategorization = cg.into();

        let expense_categorization = match budget_template::get_expense_categorization(
            db_conn_pool,
            expense_categorization.id,
        )
        .await
        {
            // TODO: Make sure to delete those newly hidden or deleted
            Ok(ec) => ec,
            Err(sqlx::Error::RowNotFound) => {
                budget_template::update_expense_categorization(
                    db_conn_pool,
                    &expense_categorization,
                )
                .await?;
                expense_categorization
            }
            Err(e) => return Err(e.into()),
        };

        expenses_categorization.push(expense_categorization);
    }

    save_categories(db_conn_pool, &categories)
        .await
        .context("failed to save categories in database")?;

    set_categories_detla(
        redis_conn,
        category_groups_with_categories_delta.server_knowledge,
    )
    .context("failed to save last known server knowledge of categories in redis")?;

    Ok((
        get_categories(db_conn_pool)
            .await
            .context("failed to get categories from database")?,
        expenses_categorization,
    ))
}

pub async fn get_categories_of_month(
    db_conn_pool: &sqlx::PgPool,
    redis_conn: &mut redis::Connection,
    ynab_client: &ynab::Client,
    month: MonthTarget,
) -> anyhow::Result<(Vec<Category>, Vec<ExpenseCategorization>)> {
    match month {
        MonthTarget::Previous | MonthTarget::Next => ynab_client
            .get_month_by_date(&DateTime::<Local>::from(month).date_naive().to_string())
            .await
            .map_err(anyhow::Error::from)
            .map(|month_detail| month_detail.categories)
            .map(|categories| (categories.clone(), vec![])), // TODO: To include expenses_categorization
        MonthTarget::Current => get_latest_categories(db_conn_pool, redis_conn, ynab_client).await,
    }
}
