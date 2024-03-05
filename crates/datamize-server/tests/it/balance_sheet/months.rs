use datamize_domain::{Month, MonthNum};
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_months().await;

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
    let response = app.get_all_months().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<Month> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 6);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_of_year_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_months(2020).await;

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
    let response = app.get_months(2023).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<Month> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 2);
}

#[derive(Debug, Clone, Serialize)]
struct CreateBody {
    month: MonthNum,
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years"))]
async fn create_returns_201_for_valid_body(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body = CreateBody {
        month: MonthNum::January,
    };
    // Act
    let response = app.create_month(2024, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn create_returns_404_when_year_does_not_exist(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body = CreateBody {
        month: MonthNum::January,
    };
    // Act
    let response = app.create_month(2020, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years", "months"))]
async fn create_returns_409_for_month_already_present(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body = CreateBody {
        month: MonthNum::January,
    };
    // Act
    let response = app.create_month(2023, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_returns_404_for_a_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_month(2020, 1).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years", "months"))]
async fn get_returns_404_for_a_non_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_month(2022, 9).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn get_returns_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_month(2022, 2).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.year, 2022);
    assert_eq!(value.month, MonthNum::February);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn delete_returns_404_for_a_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.delete_month(2020, 1).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years", "months"))]
async fn delete_returns_404_for_a_non_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.delete_month(2022, 9).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn delete_returns_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.delete_month(2022, 2).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.year, 2022);
    assert_eq!(value.month, MonthNum::February);
}
