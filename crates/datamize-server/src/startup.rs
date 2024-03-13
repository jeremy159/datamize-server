use std::sync::Arc;

use anyhow::{Context, Ok, Result};
use axum::{body::Body, routing::get, Router};
use http::{header::CONTENT_TYPE, Request};
use sqlx::PgPool;
use tokio::{net::TcpListener, signal};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::error_span;

use crate::{
    config::Settings,
    routes::{get_api_routes, health_check},
};

#[derive(Clone)]
pub struct AppState {
    pub ynab_client: Arc<ynab::Client>,
    pub db_conn_pool: PgPool,
    pub redis_conn_pool: db_redis::RedisPool,
}

pub struct Application {
    listener: TcpListener,
    port: u16,
    app: Router,
}

impl Application {
    pub async fn build(configuration: Settings, db_conn_pool: PgPool) -> Result<Self> {
        let redis_conn_pool =
            db_redis::get_connection_pool(&configuration.redis.connection_string())
                .await
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

        let listener = TcpListener::bind(address).await?;
        let socket_addr = listener.local_addr().context(format!(
            "failed to parse {}:{} to SocketAddr",
            configuration.application.host, configuration.application.port
        ))?;
        let port = socket_addr.port();

        let api_routes = get_api_routes(&app_state);

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
            .nest("/api", api_routes)
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
            listener,
            port,
            app,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self) -> Result<()> {
        tracing::debug!("listening on {}", self.listener.local_addr()?);
        // run it with hyper
        axum::serve(self.listener, self.app.into_make_service())
            .with_graceful_shutdown(Application::shutdown_signal())
            .await
            .context("failed to start hyper server")?;
        Ok(())
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        println!("signal received, starting graceful shutdown");
    }
}
