pub mod backend;
pub mod extract;

use backend::Backend;

use self::backend::OAuthBackend;

// pub type AuthSession<Backend: OAuthBackend> = axum_login::AuthSession<Backend>;
pub type AuthSession = axum_login::AuthSession<Backend>;
