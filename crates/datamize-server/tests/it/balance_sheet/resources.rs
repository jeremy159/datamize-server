use std::collections::BTreeMap;

use datamize_domain::{
    FinancialResourceType, FinancialResourceYearly, MonthNum, Uuid, Year, YearlyBalances,
};
use fake::{Dummy, Fake, Faker};
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_resources().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn get_all_returns_all_that_is_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_resources().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<FinancialResourceYearly> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 5);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_of_year_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_resources(2020).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn get_all_of_year_returns_all_that_is_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_resources(2022).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<FinancialResourceYearly> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 2);
}

#[derive(Debug, Serialize, Clone, Dummy)]
struct CreateBody {
    name: String,
    #[serde(with = "datamize_domain::string")]
    resource_type: FinancialResourceType,
    balances: BTreeMap<i32, BTreeMap<MonthNum, Option<i64>>>,
    ynab_account_ids: Option<Vec<Uuid>>,
    external_account_ids: Option<Vec<Uuid>>,
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years", "months"))]
async fn create_returns_201_for_valid_body(pool: PgPool) {
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    month_balances.insert(MonthNum::January, Some((-1000000..1000000).fake()));
    let mut balances = BTreeMap::new();
    balances.insert(2023, month_balances);

    let body = CreateBody {
        balances,
        ..Faker.fake()
    };
    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn create_returns_201_even_when_year_does_not_exist(pool: PgPool) {
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    month_balances.insert(MonthNum::January, Some((-1000000..1000000).fake()));
    let mut balances = BTreeMap::new();
    balances.insert(2023, month_balances);

    let body = CreateBody {
        balances,
        ..Faker.fake()
    };
    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years"))]
async fn create_returns_201_even_when_month_does_not_exist(pool: PgPool) {
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    month_balances.insert(MonthNum::January, Some((-1000000..1000000).fake()));
    let mut balances = BTreeMap::new();
    balances.insert(2023, month_balances);

    let body = CreateBody {
        balances,
        ..Faker.fake()
    };
    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn create_returns_409_for_resource_already_present(pool: PgPool) {
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    month_balances.insert(MonthNum::January, Some((-1000000..1000000).fake()));
    let mut balances = BTreeMap::new();
    balances.insert(2023, month_balances);

    let body = CreateBody {
        balances,
        name: "Res_Asset_Cash_Test".to_string(),
        ..Faker.fake()
    };
    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn get_returns_404_for_a_non_existing_resources(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .get_resource("47c349d4-b1e2-4057-aec5-37819791fbfa")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn get_returns_existing_resources(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .get_resource("ef6454a5-9322-4e92-9bf9-122e53b71fa7")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.base.name, "Res_Asset_Cash_Test");
    assert_eq!(value.base.resource_type.to_string(), "asset_cash");
}

#[derive(Debug, Serialize, Clone, Dummy)]
struct UpdateBody {
    id: Uuid,
    name: String,
    #[serde(with = "datamize_domain::string")]
    resource_type: FinancialResourceType,
    balances: BTreeMap<i32, BTreeMap<MonthNum, Option<i64>>>,
    ynab_account_ids: Option<Vec<Uuid>>,
    external_account_ids: Option<Vec<Uuid>>,
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn update_returns_404_for_a_non_existing_resources(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let body = UpdateBody {
        id: "47c349d4-b1e2-4057-aec5-37819791fbfa".parse().unwrap(),
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_resource("47c349d4-b1e2-4057-aec5-37819791fbfa", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn update_changes_existing_month_of_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    let new_balance = (-1000000..1000000).fake();
    month_balances.insert(MonthNum::January, Some(new_balance));
    let mut balances = BTreeMap::new();
    balances.insert(2022, month_balances);
    let body = UpdateBody {
        id: "ef6454a5-9322-4e92-9bf9-122e53b71fa7".parse().unwrap(),
        name: "Res_Asset_Cash_Test".to_string(),
        balances,
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_resource("ef6454a5-9322-4e92-9bf9-122e53b71fa7", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(
        value.get_balance(2022, MonthNum::January),
        Some(new_balance)
    );
    assert_eq!(value.get_balance(2022, MonthNum::February), Some(494498));
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn update_removes_balance_of_existing_month_of_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    month_balances.insert(MonthNum::January, None);
    let mut balances = BTreeMap::new();
    balances.insert(2022, month_balances);
    let body = UpdateBody {
        id: "ef6454a5-9322-4e92-9bf9-122e53b71fa7".parse().unwrap(),
        name: "Res_Asset_Cash_Test".to_string(),
        balances,
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_resource("ef6454a5-9322-4e92-9bf9-122e53b71fa7", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.get_balance(2022, MonthNum::January), None);
    assert_eq!(value.get_balance(2022, MonthNum::February), Some(494498));
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn update_adds_new_month_of_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    let new_balance = (-1000000..1000000).fake();
    month_balances.insert(MonthNum::June, Some(new_balance));
    let mut balances = BTreeMap::new();
    balances.insert(2022, month_balances);
    let body = UpdateBody {
        id: "ef6454a5-9322-4e92-9bf9-122e53b71fa7".parse().unwrap(),
        name: "Res_Asset_Cash_Test".to_string(),
        balances,
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_resource("ef6454a5-9322-4e92-9bf9-122e53b71fa7", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.get_balance(2022, MonthNum::June), Some(new_balance));
    assert_eq!(value.get_balance(2022, MonthNum::February), Some(494498));
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn update_does_not_change_year_net_totals_of_previous_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    let new_balance = (-1000000..1000000).fake();
    month_balances.insert(MonthNum::November, Some(new_balance));
    let mut balances = BTreeMap::new();
    balances.insert(2023, month_balances);
    let body = UpdateBody {
        id: "ef6454a5-9322-4e92-9bf9-122e53b71fa7".parse().unwrap(),
        name: "Res_Asset_Cash_Test".to_string(),
        balances,
        ..Faker.fake()
    };

    let year_2022_prev = app.get_year(2022).await;
    let year_2022_prev: Year = serde_json::from_str(&year_2022_prev.text().await.unwrap()).unwrap();

    // Act
    let response = app
        .update_resource("ef6454a5-9322-4e92-9bf9-122e53b71fa7", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let year_2022_after = app.get_year(2022).await;
    let year_2022_after: Year =
        serde_json::from_str(&year_2022_after.text().await.unwrap()).unwrap();

    assert_eq!(year_2022_after.net_totals, year_2022_prev.net_totals);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn update_does_not_put_same_net_totals_to_all_years(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let mut month_balances = BTreeMap::new();
    let new_balance = (-1000000..1000000).fake();
    month_balances.insert(MonthNum::January, Some(new_balance));
    let mut balances = BTreeMap::new();
    balances.insert(2022, month_balances);
    let body = UpdateBody {
        id: "ef6454a5-9322-4e92-9bf9-122e53b71fa7".parse().unwrap(),
        name: "Res_Asset_Cash_Test".to_string(),
        balances,
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_resource("ef6454a5-9322-4e92-9bf9-122e53b71fa7", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let year_2022: Year =
        serde_json::from_str(&app.get_year(2022).await.text().await.unwrap()).unwrap();
    let year_2023: Year =
        serde_json::from_str(&app.get_year(2023).await.text().await.unwrap()).unwrap();

    assert_ne!(
        year_2022.net_totals.assets.balance_var,
        year_2023.net_totals.assets.balance_var
    );
    assert_ne!(
        year_2022.net_totals.assets.percent_var,
        year_2023.net_totals.assets.percent_var
    );
    assert_ne!(
        year_2022.net_totals.assets.total,
        year_2023.net_totals.assets.total
    );

    assert_ne!(
        year_2022.net_totals.portfolio.balance_var,
        year_2023.net_totals.portfolio.balance_var
    );
    assert_ne!(
        year_2022.net_totals.portfolio.percent_var,
        year_2023.net_totals.portfolio.percent_var
    );
    assert_ne!(
        year_2022.net_totals.portfolio.total,
        year_2023.net_totals.portfolio.total
    );
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn delete_returns_404_for_a_non_existing_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .delete_resource("47c349d4-b1e2-4057-aec5-37819791fbfa")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn delete_returns_existing_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .delete_resource("ef6454a5-9322-4e92-9bf9-122e53b71fa7")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.base.name, "Res_Asset_Cash_Test");
    assert_eq!(value.base.resource_type.to_string(), "asset_cash");
}
