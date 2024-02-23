use axum::response::IntoResponse;
use http::StatusCode;

use crate::{error::DatamizeResult, users::AuthSession};

#[tracing::instrument(name = "Logging out and delete session cookies", skip_all)]
pub async fn logout(mut auth_session: AuthSession) -> DatamizeResult<impl IntoResponse> {
    auth_session.logout().await?;

    Ok(StatusCode::OK)
}
