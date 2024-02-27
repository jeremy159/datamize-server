use async_trait::async_trait;
use axum_login::{AuthnBackend, UserId};
use chrono::{Duration, Utc};
use datamize_domain::db::DbError;
use datamize_domain::secrecy::ExposeSecret;
use datamize_domain::{db::DynUserRepo, User};
use oauth2::basic::BasicTokenType;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, url::Url, AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use oauth2::{basic::BasicRequestTokenError, reqwest::AsyncHttpClientError};
use oauth2::{EmptyExtraTokenFields, RefreshToken, StandardTokenResponse};
use serde::Deserialize;
use ynab::UserRequests;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
}

#[derive(Clone)]
pub struct Backend {
    user_repo: DynUserRepo,
    client: BasicClient,
}

impl Backend {
    pub fn new(user_repo: DynUserRepo, client: BasicClient) -> Self {
        Self { user_repo, client }
    }

    #[tracing::instrument(name = "Get Oauth authorization URL", skip_all)]
    pub fn authorize_url(&self) -> (Url, CsrfToken) {
        self.client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read-only".to_string()))
            .url()
    }

    pub fn build_user(
        token_res: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        ynab_user: ynab::User,
    ) -> User {
        User {
            ynab_user,
            access_token: token_res.access_token().secret().to_string().into(),
            refresh_token: token_res
                .refresh_token()
                .map(|t| t.secret().to_string().into()),
            expires_at: token_res
                .expires_in()
                .and_then(|exp| Duration::from_std(exp).ok())
                .and_then(|exp| Utc::now().checked_add_signed(exp)),
        }
    }

    #[tracing::instrument(name = "Exchange refresh_token for new access_token", skip(self))]
    pub async fn refresh_token(&self, user: User) -> Result<Option<User>, BackendError> {
        if let Some(refresh_token) = user.refresh_token {
            let token_res = self
                .client
                .exchange_refresh_token(&RefreshToken::new(
                    refresh_token.expose_secret().to_string(),
                ))
                .request_async(async_http_client)
                .await
                .map_err(BackendError::OAuth2)?;

            let user = Backend::build_user(token_res, user.ynab_user);
            self.user_repo.add(&user).await?;

            return Ok(Some(user));
        }

        tracing::warn!("No refresh token found for user");
        Ok(None)
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = BackendError;

    #[tracing::instrument(name = "Exchange code for access_token", skip_all)]
    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // Ensure the CSRF state has not been tampered with.
        if creds.old_state.secret() != creds.new_state.secret() {
            return Ok(None);
        };

        // Process authorization code, expecting a token response back.
        let token_res = self
            .client
            .exchange_code(AuthorizationCode::new(creds.code))
            .request_async(async_http_client)
            .await
            .map_err(Self::Error::OAuth2)?;

        // Use access token to request user info.
        let ynab_user = ynab::Client::new(token_res.access_token().secret(), None)?
            .get_user()
            .await?;

        let user = Backend::build_user(token_res, ynab_user);
        self.user_repo.add(&user).await?;

        Ok(Some(user))
    }

    #[tracing::instrument(name = "Get user from DB", skip(self))]
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = self.user_repo.get_opt(*user_id).await?;
        let user = match user {
            None => None,
            Some(user) => {
                if let Some(expires_at) = user.expires_at {
                    if chrono::Utc::now() > expires_at {
                        self.refresh_token(user).await?
                    } else {
                        Some(user)
                    }
                } else {
                    Some(user)
                }
            }
        };
        Ok(user)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BackendError {
    #[error("Error with Database interaction")]
    DbError(#[from] DbError),
    #[error("Error in the YNAB API")]
    YnabError(#[from] ynab::Error),
    #[error("Error in the OAuth2 Process")]
    OAuth2(BasicRequestTokenError<AsyncHttpClientError>),
}
