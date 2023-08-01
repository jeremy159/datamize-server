use anyhow::Context;
use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    db::budget_providers::external::{PostgresExternalAccountRepo, RedisEncryptionKeyRepo},
    error::HttpJsonDatamizeResult,
    get_redis_conn,
    services::{balance_sheet::FinResServiceExt, budget_providers::ExternalAccountService},
    startup::AppState,
};

/// Endpoint to refresh financial resources.
/// Only resources from the current month will be refreshed by this endpoint.
/// If current month does not exists, it will create it.
/// This endpoint basically calls the YNAB api for some resources and starts a web scrapper for others.
/// Will return an array of ids for Financial Resources updated.
#[tracing::instrument(skip_all)]
pub async fn refresh_balance_sheet_resources<FRS: FinResServiceExt>(
    State(fin_res_service): State<FRS>,
    State(app_state): State<AppState>,
) -> HttpJsonDatamizeResult<Vec<Uuid>> {
    let db_conn_pool = app_state.db_conn_pool;
    let redis_conn = get_redis_conn(&app_state.redis_conn_pool)
        .context("failed to get redis connection from pool")?;
    let external_account_service = ExternalAccountService {
        external_account_repo: PostgresExternalAccountRepo { db_conn_pool },
        encryption_key_repo: RedisEncryptionKeyRepo { redis_conn },
    };
    let ynab_client = app_state.ynab_client.as_ref();

    Ok(Json(
        fin_res_service
            .refresh_fin_res(external_account_service, ynab_client)
            .await?,
    ))
}
