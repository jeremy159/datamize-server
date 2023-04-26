use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};

use anyhow::{Context, Ok, Result};
use axum::{body::Body, routing::get, Router};
use http::{header::CONTENT_TYPE, Request};
use sqlx::{PgPool, Pool, Postgres};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::error_span;

use crate::{
    config::{BugdetCalculationDataSettings, PersonSalarySettings, Settings},
    get_redis_connection_pool,
    routes::{get_api_routes, get_budget_providers_routes, health_check},
    web_scraper::get_web_scraper,
};

#[derive(Clone)]
pub struct AppState {
    pub ynab_client: Arc<ynab::Client>,
    pub db_conn_pool: Pool<Postgres>,
    pub redis_conn_pool: r2d2::Pool<redis::Client>,
    pub budget_calculation_data_settings: Arc<BugdetCalculationDataSettings>,
    pub person_salary_settings: Arc<Vec<PersonSalarySettings>>,
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
        let budget_calculation_data_settings = Arc::new(configuration.budget_calculation_data);
        let person_salary_settings = Arc::new(configuration.person_salaries);

        let app_state = AppState {
            ynab_client,
            db_conn_pool,
            redis_conn_pool,
            budget_calculation_data_settings,
            person_salary_settings,
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

        let api_routes = get_api_routes();
        let budget_providers_routes = get_budget_providers_routes();

        let origins = [
            "https://tauri.localhost"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
            "http://localhost:4300"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        ];

        let app = Router::new()
            .route("/", get(|| async { "Welcome to Datamize!" }))
            .route("/health_check", get(health_check))
            .route("/web_scraper", get(get_web_scraper)) // TODO: To remove once done with tests...
            .nest("/api", api_routes)
            .nest("/budget_providers", budget_providers_routes)
            .layer(
                CorsLayer::new()
                    .allow_origin(origins)
                    .allow_headers([CONTENT_TYPE])
                    .allow_methods([
                        axum::http::Method::GET,
                        axum::http::Method::DELETE,
                        axum::http::Method::OPTIONS,
                        axum::http::Method::HEAD,
                        axum::http::Method::POST,
                        axum::http::Method::PUT,
                    ]),
            )
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                    // We get the request id from the extensions
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "unknown".into());
                    // And then we put it along with other information into the `request` span
                    error_span!(
                        "request",
                        id = %request_id,
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                }),
            )
            .layer(RequestIdLayer)
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
