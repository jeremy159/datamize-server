use std::{net::SocketAddr, sync::Arc};

use anyhow::Ok;
use axum::{routing::get, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::{DatabaseSettings, Settings};

pub struct Application {
    socket_addr: SocketAddr,
    app: Router,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let db_conn_pool = get_connection_pool(&configuration.database);
        let redis_conn = Arc::new(
            redis::Client::open(configuration.redis.connection_string())?.get_connection()?,
        );
        let ynab_client = Arc::new(configuration.ynab_client.client());

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let socket_addr: SocketAddr = address.parse()?;

        // build our application with a single route
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .with_state(db_conn_pool)
            .with_state(redis_conn)
            .with_state(ynab_client);

        Ok(Self { socket_addr, app })
    }

    pub async fn run(self) -> Result<(), anyhow::Error> {
        // run it with hyper
        axum::Server::bind(&self.socket_addr)
            .serve(self.app.into_make_service())
            .await
            .unwrap();
        Ok(())
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
