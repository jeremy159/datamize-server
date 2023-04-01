use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use anyhow::{Context, Ok, Result};
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::{PgPool, Pool, Postgres};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    config::{RedisSettings, Settings},
    routes::{
        all_balance_sheet_months, all_balance_sheet_resources, balance_sheet_month,
        balance_sheet_months, balance_sheet_resource, balance_sheet_resources, balance_sheet_year,
        balance_sheet_years, create_balance_sheet_month, create_balance_sheet_resource,
        create_balance_sheet_year, delete_balance_sheet_month, delete_balance_sheet_resource,
        delete_balance_sheet_year, health_check, refresh_balance_sheet_resources, template_details,
        template_summary, template_transactions, update_balance_sheet_resource,
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
    port: u16,
    app: Router,
}

impl Application {
    pub async fn build(configuration: Settings, db_conn_pool: PgPool) -> Result<Self> {
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

        let listener = TcpListener::bind(address)?;
        let socket_addr = listener.local_addr().context(format!(
            "failed to parse {}:{} to SocketAddr",
            configuration.application.host, configuration.application.port
        ))?;
        let port = socket_addr.port();

        // TODO: Add tracing::instrument with request id to requests.
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
            .route(
                "/api/balance_sheet/years/:year",
                get(balance_sheet_year)
                    .put(update_balance_sheet_year)
                    .delete(delete_balance_sheet_year),
            )
            .route("/api/balance_sheet/months", get(all_balance_sheet_months))
            .route(
                "/api/balance_sheet/resources",
                get(all_balance_sheet_resources).post(create_balance_sheet_resource),
            )
            .route(
                "/api/balance_sheet/resources/:resource_id",
                get(balance_sheet_resource)
                    .put(update_balance_sheet_resource)
                    .delete(delete_balance_sheet_resource),
            )
            .route(
                "/api/balance_sheet/resources/refresh",
                post(refresh_balance_sheet_resources),
            )
            .route(
                "/api/balance_sheet/years/:year/resources",
                get(balance_sheet_resources),
            )
            .route(
                "/api/balance_sheet/years/:year/months",
                get(balance_sheet_months).post(create_balance_sheet_month),
            )
            .route(
                "/api/balance_sheet/years/:year/months/:month",
                get(balance_sheet_month).delete(delete_balance_sheet_month),
            )
            .layer(CorsLayer::permissive()) // TODO: To be more restrictive...
            .layer(TraceLayer::new_for_http())
            .with_state(app_state);

        Ok(Self {
            socket_addr,
            port,
            app,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
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
