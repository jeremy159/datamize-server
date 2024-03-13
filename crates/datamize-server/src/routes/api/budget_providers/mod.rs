use axum::Router;
use db_postgres::budget_providers::{
    external::PostgresExternalAccountRepo,
    ynab::{PostgresYnabAccountRepo, PostgresYnabPayeeRepo},
};
use db_redis::budget_providers::{
    external::RedisEncryptionKeyRepo,
    ynab::{RedisYnabAccountMetaRepo, RedisYnabPayeeMetaRepo},
};

use crate::{
    services::budget_providers::{ExternalAccountService, YnabAccountService, YnabPayeeService},
    startup::AppState,
};

mod external;
mod ynab;

use self::ynab::*;
use external::*;

pub fn get_budget_providers_routes(app_state: &AppState) -> Router<AppState> {
    let ynab_account_repo = PostgresYnabAccountRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_account_meta_repo =
        RedisYnabAccountMetaRepo::new_arced(app_state.redis_conn_pool.clone());
    let ynab_account_service = YnabAccountService::new_arced(
        ynab_account_repo,
        ynab_account_meta_repo,
        app_state.ynab_client.clone(),
    );

    let ynab_payee_repo = PostgresYnabPayeeRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_payee_meta_repo = RedisYnabPayeeMetaRepo::new_arced(app_state.redis_conn_pool.clone());
    let ynab_payee_service = YnabPayeeService::new_arced(
        ynab_payee_repo,
        ynab_payee_meta_repo,
        app_state.ynab_client.clone(),
    );

    let external_account_repo =
        PostgresExternalAccountRepo::new_arced(app_state.db_conn_pool.clone());
    let encryption_key_repo = RedisEncryptionKeyRepo::new_arced(app_state.redis_conn_pool.clone());
    let external_acount_service =
        ExternalAccountService::new_arced(external_account_repo, encryption_key_repo);

    Router::new()
        .nest(
            "/ynab",
            get_ynab_routes(ynab_account_service, ynab_payee_service),
        )
        .nest("/external", get_external_routes(external_acount_service))
}
