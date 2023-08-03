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

impl FromRef<AppState> for YnabAccountService<PostgresYnabAccountRepo, RedisYnabAccountMetaRepo> {
    fn from_ref(state: &AppState) -> Self {
        Self {
            ynab_account_repo: PostgresYnabAccountRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
            ynab_account_meta_repo: RedisYnabAccountMetaRepo {
                redis_conn: state.redis_conn.clone(),
            },
            ynab_client: state.ynab_client.clone(),
        }
    }
}

impl FromRef<AppState> for YnabPayeeService<PostgresYnabPayeeRepo, RedisYnabPayeeMetaRepo> {
    fn from_ref(state: &AppState) -> Self {
        Self {
            ynab_payee_repo: PostgresYnabPayeeRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
            ynab_payee_meta_repo: RedisYnabPayeeMetaRepo {
                redis_conn: state.redis_conn.clone(),
            },
            ynab_client: state.ynab_client.clone(),
        }
    }
}

pub fn get_ynab_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/accounts",
            get(get_ynab_accounts::<
                YnabAccountService<PostgresYnabAccountRepo, RedisYnabAccountMetaRepo>,
            >),
        )
        .route(
            "/payees",
            get(get_ynab_payees::<YnabPayeeService<PostgresYnabPayeeRepo, RedisYnabPayeeMetaRepo>>),
        )
}
