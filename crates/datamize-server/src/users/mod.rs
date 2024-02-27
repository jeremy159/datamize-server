pub mod backend;
pub mod extract;

use backend::Backend;

pub type AuthSession = axum_login::AuthSession<Backend>;
