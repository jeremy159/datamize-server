use async_trait::async_trait;
use axum_login::{AuthnBackend, UserId};
use chrono::{Duration, Utc};
use datamize_domain::db::DbError;
use datamize_domain::secrecy::ExposeSecret;
use datamize_domain::{db::DynUserRepo, User};
use oauth2::RefreshToken;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, url::Url, AuthorizationCode, CsrfToken, Scope,
    TokenResponse,
};
use oauth2::{basic::BasicRequestTokenError, reqwest::AsyncHttpClientError};
use serde::Deserialize;
use ynab::UserRequests;

#[async_trait]
pub trait OAuthBackend: AuthnBackend {
    fn authorize_url(&self) -> (Url, CsrfToken);
    async fn refresh_token(&self, user: Self::User) -> Result<Option<Self::User>, Self::Error>;
}

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

            let user = User {
                ynab_user: user.ynab_user,
                access_token: token_res.access_token().secret().to_string().into(),
                refresh_token: token_res
                    .refresh_token()
                    .map(|t| t.secret().to_string().into()),
                expires_at: token_res
                    .expires_in()
                    .and_then(|exp| Duration::from_std(exp).ok())
                    .and_then(|exp| Utc::now().checked_add_signed(exp)),
            };
            self.user_repo.add(&user).await?;

            return Ok(Some(user));
        }

        tracing::warn!("No refresh token found for user");
        Ok(None)
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
        // TODO: We need to either only set access_token here or find a way to save this client to reuse it.
        let ynab_user = ynab::Client::new(token_res.access_token().secret(), None)?
            .get_user()
            .await?;

        let user = User {
            ynab_user,
            access_token: token_res.access_token().secret().to_string().into(),
            refresh_token: token_res
                .refresh_token()
                .map(|t| t.secret().to_string().into()),
            expires_at: token_res
                .expires_in()
                .and_then(|exp| Duration::from_std(exp).ok())
                .and_then(|exp| Utc::now().checked_add_signed(exp)),
        };
        self.user_repo.add(&user).await?;

        Ok(Some(user))
    }

    #[tracing::instrument(name = "Get user from DB", skip(self))]
    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.user_repo.get_opt(*user_id).await?)
    }
}
