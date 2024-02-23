use axum::extract::Query;
use axum_login::tower_sessions::Session;
use oauth2::CsrfToken;
use serde::Deserialize;

use crate::{
    error::{AppError, DatamizeResult},
    users::{backend::Credentials, AuthSession},
};

use super::CSRF_STATE_KEY;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}

#[tracing::instrument(
    name = "Get code returned from YNAB and exchange it for access_token",
    skip_all
)]
pub async fn callback(
    mut auth_session: AuthSession,
    session: Session,
    Query(AuthzResp {
        code,
        state: new_state,
    }): Query<AuthzResp>,
) -> DatamizeResult<()> {
    let Some(old_state) = session
        .get(CSRF_STATE_KEY)
        .await
        .map_err(|_| AppError::MissingCsrfToken)?
    else {
        return Err(AppError::MissingCsrfToken);
    };

    let creds = Credentials {
        code,
        old_state,
        new_state,
    };

    let Some(user) = auth_session
        .authenticate(creds)
        .await
        .map_err(AppError::OauthBackendError)?
    else {
        return Err(AppError::Unauthorized);
    };

    auth_session
        .login(&user)
        .await
        .map_err(AppError::OauthBackendError)?;

    Ok(())
}
