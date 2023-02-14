use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Ok, Result};
use axum::{
    routing::{get, put},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    config::{DatabaseSettings, RedisSettings, Settings},
    routes::{
        balance_sheet_month, balance_sheet_months, balance_sheet_year, balance_sheet_years,
        create_balance_sheet_month, create_balance_sheet_year, health_check, template_details,
        template_summary, template_transactions, update_balance_sheet_month,
        update_balance_sheet_year,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub ynab_client: Arc<ynab::Client>,
    pub db_conn_pool: Pool<Postgres>,
    pub redis_conn_pool: r2d2::Pool<redis::Client>,
}

pub struct Application {
    socket_addr: SocketAddr,
    app: Router,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self> {
        let db_conn_pool = get_connection_pool(&configuration.database);
        let redis_conn_pool = get_redis_connection_pool(&configuration.redis)
            .context("failed to get redis connection pool")?;
        let ynab_client = Arc::new(configuration.ynab_client.client());

        let app_state = AppState {
            ynab_client,
            db_conn_pool,
            redis_conn_pool,
        };

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let socket_addr: SocketAddr = address.parse().context(format!(
            "failed to parse {}:{} to SocketAddr",
            configuration.application.host, configuration.application.port
        ))?;

        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/health_check", get(health_check))
            .route("/api/template/details", get(template_details))
            .route("/api/template/summary", get(template_summary))
            .route("/api/template/transactions", get(template_transactions))
            .route(
                "/api/balance_sheet/years",
                get(balance_sheet_years).post(create_balance_sheet_year),
            )
            .route("/api/balance_sheet/years/:year", get(balance_sheet_year))
            .route(
                "/api/balance_sheet/years/:year",
                put(update_balance_sheet_year),
            )
            .route(
                "/api/balance_sheet/years/:year/months",
                get(balance_sheet_months).post(create_balance_sheet_month),
            )
            .route(
                "/api/balance_sheet/years/:year/months/:month",
                get(balance_sheet_month).put(update_balance_sheet_month),
            )
            .layer(CorsLayer::permissive()) // TODO: To be more restrictive...
            .layer(TraceLayer::new_for_http())
            .with_state(app_state);

        Ok(Self { socket_addr, app })
    }

    pub async fn run(self) -> Result<()> {
        tracing::debug!("listening on {}", self.socket_addr);
        // run it with hyper
        axum::Server::bind(&self.socket_addr)
            .serve(self.app.into_make_service())
            .await
            .context("failed to start hyper server")?;
        Ok(())
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn get_redis_connection_pool(
    configuration: &RedisSettings,
) -> Result<r2d2::Pool<redis::Client>> {
    let redis_client = redis::Client::open(configuration.connection_string())
        .context("failed to establish connection to the redis instance")?;
    Ok(r2d2::Pool::new(redis_client).context("failed to create pool of redis connections")?)
}

pub fn get_redis_conn(
    pool: &r2d2::Pool<redis::Client>,
) -> Result<r2d2::PooledConnection<redis::Client>, r2d2::Error> {
    pool.get()
}
