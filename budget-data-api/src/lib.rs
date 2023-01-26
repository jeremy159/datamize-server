pub mod config;
mod data_builder;
mod db;
pub mod web_scraper;

use std::collections::HashMap;

use crate::config::BudgetDataConfig;
use anyhow::Result;
use db::{
    get_categories, get_categories_delta, get_category_by_id, get_scheduled_transactions,
    get_scheduled_transactions_delta, save_categories, save_scheduled_transactions,
    set_categories_detla, set_scheduled_transactions_delta,
};
use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::StreamExt;
use futures::try_join;
use sqlx::{PgPool, Pool, Postgres};
use ynab::Client;

pub type BudgetDetails = data_builder::types::BudgetDetails;
pub type CommonExpanseEstimationPerPerson = data_builder::types::CommonExpanseEstimationPerPerson;
pub type ScheduledTransactionsDistribution = data_builder::types::ScheduledTransactionsDistribution;

pub async fn get_budget_details() -> Result<BudgetDetails> {
    let (config, db_conn_pool, mut redis_conn) = get_config_and_database_connection().await?;

    let saved_categories_delta = get_categories_delta(&mut redis_conn);
    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(&mut redis_conn);

    let client = Client::new(&config.ynab_pat, &config.ynab_base_url)?;

    let (category_groups_with_categories_delta, scheduled_transactions_delta) = try_join!(
        client.get_categories_delta(saved_categories_delta),
        client.get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
    )?;

    let categories = category_groups_with_categories_delta
        .category_groups
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect::<Vec<_>>();

    save_categories(&db_conn_pool, &categories).await?;

    set_categories_detla(
        &mut redis_conn,
        category_groups_with_categories_delta.server_knowledge,
    )?;

    save_scheduled_transactions(
        &db_conn_pool,
        &scheduled_transactions_delta.scheduled_transactions,
    )
    .await?;

    set_scheduled_transactions_delta(
        &mut redis_conn,
        scheduled_transactions_delta.server_knowledge,
    )?;

    let (saved_categories, saved_scheduled_transactions) = try_join!(
        get_categories(&db_conn_pool),
        get_scheduled_transactions(&db_conn_pool)
    )?;

    data_builder::budget_details(
        &saved_categories,
        &saved_scheduled_transactions,
        &config.budget_calculation_data,
    )
    .await
}

pub async fn get_proportional_split_common_expanses(
) -> Result<Vec<CommonExpanseEstimationPerPerson>> {
    let (config, db_conn_pool, mut redis_conn) = get_config_and_database_connection().await?;

    let saved_categories_delta = get_categories_delta(&mut redis_conn);
    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(&mut redis_conn);

    let client = Client::new(&config.ynab_pat, &config.ynab_base_url)?;

    let (category_groups_with_categories_delta, scheduled_transactions_delta) = try_join!(
        client.get_categories_delta(saved_categories_delta),
        client.get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
    )?;

    let categories = category_groups_with_categories_delta
        .category_groups
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect::<Vec<_>>();

    save_categories(&db_conn_pool, &categories).await?;

    set_categories_detla(
        &mut redis_conn,
        category_groups_with_categories_delta.server_knowledge,
    )?;

    save_scheduled_transactions(
        &db_conn_pool,
        &scheduled_transactions_delta.scheduled_transactions,
    )
    .await?;

    set_scheduled_transactions_delta(
        &mut redis_conn,
        scheduled_transactions_delta.server_knowledge,
    )?;

    let (saved_categories, saved_scheduled_transactions) = try_join!(
        get_categories(&db_conn_pool),
        get_scheduled_transactions(&db_conn_pool)
    )?;

    let budget_details = data_builder::budget_details(
        &saved_categories,
        &saved_scheduled_transactions,
        &config.budget_calculation_data,
    )
    .await?;

    let data = data_builder::common_expanses(&budget_details, &saved_scheduled_transactions)?;

    Ok(data)
}

pub async fn get_scheduled_transactions_distribution() -> Result<ScheduledTransactionsDistribution>
{
    let (config, db_conn_pool, mut redis_conn) = get_config_and_database_connection().await?;

    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(&mut redis_conn);

    let client = Client::new(&config.ynab_pat, &config.ynab_base_url)?;

    let scheduled_transactions_delta = client
        .get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
        .await?;

    save_scheduled_transactions(
        &db_conn_pool,
        &scheduled_transactions_delta.scheduled_transactions,
    )
    .await?;

    set_scheduled_transactions_delta(
        &mut redis_conn,
        scheduled_transactions_delta.server_knowledge,
    )?;

    let saved_scheduled_transactions = get_scheduled_transactions(&db_conn_pool).await?;

    let category_ids =
        data_builder::utils::get_subtransactions_category_ids(&saved_scheduled_transactions);

    let mut category_id_to_name_map: HashMap<uuid::Uuid, String> = HashMap::new();

    let categories_stream = category_ids
        .iter()
        .map(|cat_id| get_category_by_id(&db_conn_pool, cat_id))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await;

    for (index, category) in categories_stream.into_iter().enumerate() {
        let category = match category? {
            Some(cat) => cat,
            None => {
                client
                    .get_category_by_id(&category_ids[index].to_string())
                    .await?
            }
        };
        category_id_to_name_map.insert(category.id, category.name);
    }

    let data = data_builder::scheduled_transactions(
        &saved_scheduled_transactions,
        &category_id_to_name_map,
    )
    .await?;

    Ok(data)
}

async fn get_config_and_database_connection(
) -> Result<(BudgetDataConfig, Pool<Postgres>, redis::Connection)> {
    let config = BudgetDataConfig::build();
    let db_conn_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let redis_conn = redis::Client::open(config.redis.connection_string())?.get_connection()?;

    Ok((config, db_conn_pool, redis_conn))
}
