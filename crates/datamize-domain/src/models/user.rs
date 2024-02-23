use axum_login::AuthUser;
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Deserialize, Debug)]
pub struct User {
    pub ynab_user: ynab::User,
    pub access_token: Secret<String>,
    pub refresh_token: Option<Secret<String>>,
    /// The UTC timestampt at which the access token becomes unusable.
    // TODO: Create background task to automatically refresh access_token once expired.
    pub expires_at: Option<DateTime<Utc>>,
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.ynab_user.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        // TODO: Do we want to use that or simply return empty slice to not expose the access_token?
        self.access_token.expose_secret().as_bytes()
    }
}
