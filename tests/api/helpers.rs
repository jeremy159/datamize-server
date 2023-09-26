use std::fmt::Display;

use chrono::{Datelike, NaiveDate, Utc};
use datamize::{
    config, get_redis_connection_manager,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use fake::{faker::chrono::en::Date, Fake, Faker};
use once_cell::sync::Lazy;
use redis::aio::ConnectionManager;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::MockServer;

use crate::dummy_types::{
    DummyFinancialResource, DummyMonthNum, DummyNetTotal, DummyNetTotalType, DummyResourceCategory,
    DummyResourceType,
};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "warn".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub redis_pool: ConnectionManager,
    pub api_client: reqwest::Client,
    pub ynab_server: MockServer,
    pub ynab_client: ynab::Client,
}

impl TestApp {
    pub async fn get_years_summary(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/balance_sheet/years", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_year<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .post(&format!("{}/api/balance_sheet/years", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_year<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_year<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .delete(&format!(
                "{}/api/balance_sheet/years/{}",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_months<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}/months",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_month<Y, B>(&self, year: Y, body: &B) -> reqwest::Response
    where
        Y: Display,
        B: Serialize,
    {
        self.api_client
            .post(&format!(
                "{}/api/balance_sheet/years/{}/months",
                &self.address, year
            ))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_month<Y, M>(&self, year: Y, month: M) -> reqwest::Response
    where
        Y: Display,
        M: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}/months/{}",
                &self.address, year, month
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_month<Y, M>(&self, year: Y, month: M) -> reqwest::Response
    where
        Y: Display,
        M: Display,
    {
        self.api_client
            .delete(&format!(
                "{}/api/balance_sheet/years/{}/months/{}",
                &self.address, year, month
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn refresh_resources(&self) -> reqwest::Response {
        self.api_client
            .post(&format!(
                "{}/api/balance_sheet/resources/refresh",
                &self.address
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_all_months(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/balance_sheet/months", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_all_resources(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/balance_sheet/resources", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_resources<Y>(&self, year: Y) -> reqwest::Response
    where
        Y: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}/resources",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_resource<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .post(&format!("{}/api/balance_sheet/resources", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_resource<R>(&self, res_id: R) -> reqwest::Response
    where
        R: Display,
    {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/resources/{}",
                &self.address, res_id
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn update_resource<R, B>(&self, res_id: R, body: &B) -> reqwest::Response
    where
        R: Display,
        B: Serialize,
    {
        self.api_client
            .put(&format!(
                "{}/api/balance_sheet/resources/{}",
                &self.address, res_id
            ))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn delete_resource<R>(&self, res_id: R) -> reqwest::Response
    where
        R: Display,
    {
        self.api_client
            .delete(&format!(
                "{}/api/balance_sheet/resources/{}",
                &self.address, res_id
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn insert_year(&self, year: i32) -> Uuid {
        let year_id = uuid::Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_years (id, year, refreshed_at)
            VALUES ($1, $2, $3);
            "#,
            year_id,
            year,
            Utc::now(),
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert a year.");

        year_id
    }

    pub async fn insert_year_net_total(
        &self,
        year_id: Uuid,
        net_type: DummyNetTotalType,
    ) -> DummyNetTotal {
        let net_total = DummyNetTotal {
            net_type: net_type.clone(),
            ..Faker.fake()
        };

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, year_id)
            VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            net_total.id,
            net_type.to_string(),
            net_total.total as i64,
            net_total.percent_var,
            net_total.balance_var,
            year_id,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert net totals of a year.");

        net_total
    }

    pub async fn insert_month(&self, year_id: Uuid, month: i16) -> Uuid {
        let month_id = uuid::Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_months (id, month, year_id)
            VALUES ($1, $2, $3);
            "#,
            month_id,
            month,
            year_id,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert month.");

        month_id
    }

    pub async fn insert_random_month(&self, year_id: Uuid) -> (Uuid, DummyMonthNum) {
        let month_id = uuid::Uuid::new_v4();
        let month: DummyMonthNum = Date().fake::<NaiveDate>().month().try_into().unwrap();

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_months (id, month, year_id)
            VALUES ($1, $2, $3);
            "#,
            month_id,
            month as i16,
            year_id,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert month.");

        (month_id, month)
    }

    pub async fn insert_month_net_total(
        &self,
        month_id: Uuid,
        net_type: DummyNetTotalType,
    ) -> DummyNetTotal {
        let net_total = DummyNetTotal {
            net_type: net_type.clone(),
            ..Faker.fake()
        };

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_months (id, type, total, percent_var, balance_var, month_id)
            VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            net_total.id,
            net_total.net_type.to_string(),
            net_total.total as i64,
            net_total.percent_var,
            net_total.balance_var as i64,
            month_id,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert net totals of a month.");

        net_total
    }

    pub async fn insert_financial_resource(
        &self,
        month_id: Uuid,
        category: DummyResourceCategory,
        res_type: DummyResourceType,
    ) -> DummyFinancialResource {
        self.insert_financial_resource_with_balance(month_id, Faker.fake(), category, res_type)
            .await
    }

    pub async fn insert_financial_resource_with_balance(
        &self,
        month_id: Uuid,
        balance: i64,
        category: DummyResourceCategory,
        res_type: DummyResourceType,
    ) -> DummyFinancialResource {
        self.insert_financial_resource_with_balance_and_ynab_account_ids(
            month_id, balance, None, category, res_type,
        )
        .await
    }

    pub async fn insert_financial_resource_with_balance_and_ynab_account_ids(
        &self,
        month_id: Uuid,
        balance: i64,
        ynab_account_ids: Option<Vec<Uuid>>,
        category: DummyResourceCategory,
        res_type: DummyResourceType,
    ) -> DummyFinancialResource {
        let resource = DummyFinancialResource {
            category,
            resource_type: res_type,
            balance,
            ynab_account_ids,
            ..Faker.fake()
        };

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, category, type, editable, ynab_account_ids)
            VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            resource.id,
            resource.name,
            resource.category.to_string(),
            resource.resource_type.to_string(),
            resource.editable,
            resource
                .ynab_account_ids
                .as_ref()
                .map(|accounts| accounts.as_slice()),
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert financial resource of a month.");

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
            VALUES ($1, $2, $3)
            ON CONFLICT (resource_id, month_id) DO UPDATE SET
            balance = EXCLUDED.balance;
            "#,
            resource.id,
            month_id,
            resource.balance,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert balance of financial resource.");

        resource
    }

    pub async fn insert_financial_resource_with_id_in_month(
        &self,
        month_id: Uuid,
        id: Uuid,
    ) -> i64 {
        let balance: i64 = Faker.fake();

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
            VALUES ($1, $2, $3)
            ON CONFLICT (resource_id, month_id) DO UPDATE SET
            balance = EXCLUDED.balance;
            "#,
            id,
            month_id,
            balance,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert balance of financial resource.");

        balance
    }
}

pub async fn spawn_app(db_pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for YNAB's API
    let ynab_server = MockServer::start().await;

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = config::Settings::build().expect("Failed to read configuration.");
        // Use a random OS port
        c.application.port = 0;
        // Use the mock server as ynab API
        c.ynab_client.base_url = ynab_server.uri();
        c
    };

    // Launch the application as a background task
    let application = Application::build(configuration.clone(), db_pool.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run());

    // Give time for the app to start before sending in requests. Not ideal at all...
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool,
        redis_pool: get_redis_connection_manager(&configuration.redis)
            .await
            .expect("Failed to start connection to redis."),
        api_client: client,
        ynab_server,
        ynab_client: configuration.ynab_client.client(),
    }
}
