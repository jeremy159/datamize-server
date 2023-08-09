mod accounts;
mod payees;

use accounts::*;
use axum::{extract::FromRef, routing::get, Router};
use payees::*;

use crate::{
    db::budget_providers::ynab::{
        PostgresYnabAccountRepo, PostgresYnabPayeeRepo, RedisYnabAccountMetaRepo,
        RedisYnabPayeeMetaRepo,
    },
    services::budget_providers::{YnabAccountService, YnabPayeeService},
    startup::AppState,
};

impl FromRef<AppState> for YnabAccountService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            ynab_account_repo: Box::new(PostgresYnabAccountRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            ynab_account_meta_repo: Box::new(RedisYnabAccountMetaRepo {
                redis_conn: state.redis_conn.clone(),
            }),
            ynab_client: state.ynab_client.clone(),
        }
    }
}

impl FromRef<AppState> for YnabPayeeService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            ynab_payee_repo: Box::new(PostgresYnabPayeeRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            ynab_payee_meta_repo: Box::new(RedisYnabPayeeMetaRepo {
                redis_conn: state.redis_conn.clone(),
            }),
            ynab_client: state.ynab_client.clone(),
        }
    }
}

pub fn get_ynab_routes() -> Router<AppState> {
    Router::new()
        .route("/accounts", get(get_ynab_accounts))
        .route("/payees", get(get_ynab_payees))
}
