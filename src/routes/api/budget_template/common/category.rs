use std::collections::HashSet;

use anyhow::Context;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use ynab::types::{Category, CategoryGroup};

use crate::{
    db::{
        budget_providers::ynab::{
            del_categories_detla, get_categories, get_categories_delta, get_categories_last_saved,
            save_categories, set_categories_detla, set_categories_last_saved,
        },
        budget_template,
    },
    models::budget_template::{ExpenseCategorization, MonthTarget},
};

pub async fn get_categories_of_month(
    db_conn_pool: &sqlx::PgPool,
    redis_conn: &mut redis::Connection,
    ynab_client: &ynab::Client,
    month: MonthTarget,
) -> anyhow::Result<(Vec<Category>, Vec<ExpenseCategorization>)> {
    match month {
        MonthTarget::Previous | MonthTarget::Next => {
            let categories = ynab_client
                .get_month_by_date(&DateTime::<Local>::from(month).date_naive().to_string())
                .await
                .map_err(anyhow::Error::from)
                .map(|month_detail| month_detail.categories)?;

            let expenses_categorization =
                get_expenses_categorization(db_conn_pool, categories.clone()).await?;

            Ok((categories, expenses_categorization))
        }
        MonthTarget::Current => get_latest_categories(db_conn_pool, redis_conn, ynab_client).await,
    }
}

pub async fn get_latest_categories(
    db_conn_pool: &sqlx::PgPool,
    redis_conn: &mut redis::Connection,
    ynab_client: &ynab::Client,
) -> anyhow::Result<(Vec<Category>, Vec<ExpenseCategorization>)> {
    let current_date = Local::now().date_naive();
    if let Some(last_saved) = get_categories_last_saved(redis_conn) {
        let last_saved_date: NaiveDate = last_saved.parse()?;
        if current_date.month() != last_saved_date.month() {
            // Discard knowledge_server when changing month.
            del_categories_detla(redis_conn)?;
            set_categories_last_saved(redis_conn, current_date.to_string())?;
        }
    } else {
        set_categories_last_saved(redis_conn, current_date.to_string())?;
    }
    let saved_categories_delta = get_categories_delta(redis_conn);

    let category_groups_with_categories_delta = ynab_client
        .get_categories_delta(saved_categories_delta)
        .await
        .context("failed to get categories from ynab's API")?;

    let (category_groups, categories): (Vec<CategoryGroup>, Vec<Vec<Category>>) =
        category_groups_with_categories_delta
            .category_groups
            .into_iter()
            .map(|cg| {
                (
                    CategoryGroup {
                        id: cg.id,
                        name: cg.name,
                        hidden: cg.hidden,
                        deleted: cg.deleted,
                    },
                    cg.categories,
                )
            })
            .unzip();

    let categories = categories.into_iter().flatten().collect::<Vec<_>>();

    let expenses_categorization =
        get_expenses_categorization(db_conn_pool, category_groups).await?;

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

async fn get_expenses_categorization<T: TryInto<ExpenseCategorization>>(
    db_conn_pool: &sqlx::PgPool,
    categories: Vec<T>,
) -> anyhow::Result<Vec<ExpenseCategorization>> {
    let mut expenses_categorization_set = HashSet::<ExpenseCategorization>::new();

    let expenses_categorization = categories
        .into_iter()
        .flat_map(TryInto::try_into)
        .collect::<Vec<_>>();

    for ec in expenses_categorization {
        if !expenses_categorization_set.contains(&ec) {
            let expense_categorization =
                match budget_template::get_expense_categorization(db_conn_pool, ec.id).await {
                    // TODO: Make sure to delete those newly hidden or deleted (Applies to all data coming from YNAB)
                    Ok(ec) => ec,
                    Err(sqlx::Error::RowNotFound) => {
                        budget_template::update_expense_categorization(db_conn_pool, &ec).await?;
                        ec
                    }
                    Err(e) => return Err(e.into()),
                };

            expenses_categorization_set.insert(expense_categorization);
        }
    }

    Ok(expenses_categorization_set.into_iter().collect())
}
