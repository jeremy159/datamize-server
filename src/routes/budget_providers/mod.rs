use axum::Router;

use crate::{
    services::budget_providers::{ExternalAccountService, YnabAccountService, YnabPayeeService},
    startup::AppState,
};

mod external;
mod ynab;

use self::ynab::*;
use external::*;

pub fn get_budget_providers_routes(app_state: &AppState) -> Router<AppState> {
    let ynab_account_service = YnabAccountService::new_boxed(
        app_state.db_conn_pool.clone(),
        app_state.redis_conn.clone(),
        app_state.ynab_client.clone(),
    );

    let ynab_payee_service = YnabPayeeService::new_boxed(
        app_state.db_conn_pool.clone(),
        app_state.redis_conn.clone(),
        app_state.ynab_client.clone(),
    );

    let external_acount_service = ExternalAccountService::new_boxed(
        app_state.db_conn_pool.clone(),
        app_state.redis_conn.clone(),
    );

    Router::new()
        .nest(
            "/ynab",
            get_ynab_routes(ynab_account_service, ynab_payee_service),
        )
        .nest("/external", get_external_routes(external_acount_service))
}
