use axum::{extract::FromRef, routing::get, Router};

use crate::{
    db::budget_providers::external::{PostgresExternalAccountRepo, RedisEncryptionKeyRepo},
    services::budget_providers::ExternalAccountService,
    startup::AppState,
};

mod accounts;

use accounts::*;

impl FromRef<AppState> for ExternalAccountService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            external_account_repo: Box::new(PostgresExternalAccountRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            encryption_key_repo: Box::new(RedisEncryptionKeyRepo {
                redis_conn: state.redis_conn.clone(),
            }),
        }
    }
}

pub fn get_external_routes() -> Router<AppState> {
    Router::new().route("/accounts", get(get_external_accounts))
}
