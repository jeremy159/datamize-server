use axum::response::{IntoResponse, Redirect};
use axum_login::tower_sessions::Session;

use crate::{error::DatamizeResult, users::AuthSession};

use super::CSRF_STATE_KEY;

#[tracing::instrument(name = "Starting the Oauth flow for YNAB", skip_all)]
pub async fn login(
    auth_session: AuthSession,
    session: Session,
) -> DatamizeResult<impl IntoResponse> {
    let (auth_url, csrf_state) = auth_session.backend.authorize_url();

    session.insert(CSRF_STATE_KEY, csrf_state.secret()).await?;

    Ok(Redirect::to(auth_url.as_str()))
}
