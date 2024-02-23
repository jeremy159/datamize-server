use async_trait::async_trait;
use axum::extract::FromRequestParts;
use datamize_domain::User;
use http::request::Parts;

use crate::error::AppError;

use super::AuthSession;

/// This is the Extension that should be used whenever an endpoint needs to be secured.
#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    // If anything goes wrong or no session is found, return 401 so client can handle properly.
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .expect("unexpected error getting AuthSession");

        Ok(auth_session
            .user
            .map(AuthUser)
            .ok_or(AppError::Unauthorized)?)
    }
}
