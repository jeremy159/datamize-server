mod callback;
mod login;
mod logout;

pub use callback::*;
pub use login::*;
pub use logout::*;

use axum::{routing::get, Router};
use db_postgres::user::PostgresUserRepo;
use oauth2::basic::BasicClient;

use crate::{startup::AppState, users::backend::Backend};

pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";

pub fn get_oauth_routes<S: Clone + Send + Sync + 'static>(
    app_state: &AppState,
    oauth_client: BasicClient,
) -> (Router<S>, Backend) {
    let user_repo = PostgresUserRepo::new_arced(app_state.db_conn_pool.clone());
    let backend = Backend::new(user_repo, oauth_client);

    (
        Router::new()
            .merge(get_callback_route())
            .merge(get_login_route())
            .merge(get_logout_route()),
        backend,
    )
}

fn get_callback_route<S: Clone + Sync + Send + 'static>() -> Router<S> {
    Router::new().route("/callback", get(callback))
}

fn get_login_route<S: Clone + Sync + Send + 'static>() -> Router<S> {
    Router::new().route("/login", get(login))
}

fn get_logout_route<S: Clone + Sync + Send + 'static>() -> Router<S> {
    Router::new().route("/logout", get(logout))
}
