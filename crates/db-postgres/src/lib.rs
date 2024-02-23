pub mod balance_sheet;
pub mod budget_providers;
pub mod budget_template;
pub mod user;

use sqlx::PgPool;
pub use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
    Error,
};

pub fn get_connection_pool(options: PgConnectOptions) -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(options)
}
