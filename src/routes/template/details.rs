use axum::{extract::State, response::IntoResponse, Json};
use futures::try_join;

use crate::{
    db::{
        get_categories, get_categories_delta, get_scheduled_transactions,
        get_scheduled_transactions_delta, save_categories, save_scheduled_transactions,
        set_categories_detla, set_scheduled_transactions_delta,
    },
    startup::{get_redis_conn, AppState},
};

// TODO: Handle errors better, maybe returning a 500...
/// Returns a budget template details
pub async fn template_details(State(app_state): State<AppState>) -> impl IntoResponse {
    let ynab_client = app_state.ynab_client.as_ref();
    let db_conn_pool = app_state.db_conn_pool;
    let mut redis_conn = get_redis_conn(&app_state.redis_conn_pool);

    let saved_categories_delta = get_categories_delta(&mut redis_conn);
    let saved_scheduled_transactions_delta = get_scheduled_transactions_delta(&mut redis_conn);

    let (category_groups_with_categories_delta, scheduled_transactions_delta) = try_join!(
        ynab_client.get_categories_delta(saved_categories_delta),
        ynab_client.get_scheduled_transactions_delta(saved_scheduled_transactions_delta)
    )
    .unwrap();

    let categories = category_groups_with_categories_delta
        .category_groups
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect::<Vec<_>>();

    save_categories(&db_conn_pool, &categories).await.unwrap();

    set_categories_detla(
        &mut redis_conn,
        category_groups_with_categories_delta.server_knowledge,
    )
    .unwrap();

    save_scheduled_transactions(
        &db_conn_pool,
        &scheduled_transactions_delta.scheduled_transactions,
    )
    .await
    .unwrap();

    set_scheduled_transactions_delta(
        &mut redis_conn,
        scheduled_transactions_delta.server_knowledge,
    )
    .unwrap();

    let (saved_categories, saved_scheduled_transactions) = try_join!(
        get_categories(&db_conn_pool),
        get_scheduled_transactions(&db_conn_pool)
    )
    .unwrap();

    let data =
        budget_data_api::build_budget_details(&saved_categories, &saved_scheduled_transactions)
            .unwrap();

    Json(data)
}
