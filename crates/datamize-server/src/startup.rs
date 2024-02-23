use std::sync::Arc;

use anyhow::{Context, Ok, Result};
use axum::{body::Body, routing::get, Router};
use axum_login::{
    tower_sessions::{
        cookie::{time::Duration, SameSite},
        Expiry, SessionManagerLayer,
    },
    AuthManagerLayerBuilder,
};
use db_redis::towser_sessions_store::RedisStore;
use http::{header::CONTENT_TYPE, Request};
use sqlx::PgPool;
use tokio::{net::TcpListener, signal};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_request_id::{RequestId, RequestIdLayer};
use tracing::error_span;

use crate::{
    config::Settings,
    routes::{get_api_routes, get_budget_providers_routes, get_oauth_routes, health_check},
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
        let ynab_client = Arc::new(configuration.ynab_client.clone().client()); // TODO: To remove, will need one ynab_client per different user, since access_token is only known once we identify him.
        let ynab_oauth_client = configuration.ynab_client.oauth_client()?;

        let app_state = AppState {
            ynab_client,
            db_conn_pool,
            redis_conn_pool: redis_conn_pool.clone(),
        };

        // Session layer.
        //
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let session_store = RedisStore::new(redis_conn_pool);
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let (oauth_routes, backend) = get_oauth_routes(&app_state, ynab_oauth_client);
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

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
        let budget_providers_routes = get_budget_providers_routes(&app_state);

        let origins = [
            "https://tauri.localhost"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
            "http://localhost:4300"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
            "http://localhost:8080"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        ];

        // TODO: Implement oauth with YNAB. Check https://github.com/tokio-rs/axum/blob/main/examples/oauth/src/main.rs as an example
        // TODO: Also check https://github.com/maxcountryman/axum-login/blob/main/examples/oauth2/src/main.rs
        let app = Router::new()
            .nest("/api", api_routes)
            .nest("/budget_providers", budget_providers_routes)
            .nest("/oauth", oauth_routes)
            .route("/", get(|| async { "Welcome to Datamize!" }))
            .route("/health_check", get(health_check))
            .layer(auth_layer)
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
