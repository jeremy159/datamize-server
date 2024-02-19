use chrono::{Datelike, NaiveDate};
use datamize_domain::Year;
use fake::{faker::chrono::en::Date, Fake};
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_years().await;

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
    let response = app.get_years().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<Year> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 3);
}

#[derive(Debug, Clone, Serialize)]
struct CreateBody {
    year: i32,
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn create_returns_201_for_valid_body(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body = CreateBody {
        year: Date().fake::<NaiveDate>().year(),
    };
    // Act
    let response = app.create_year(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("years"))]
async fn create_returns_409_for_year_already_present(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body = CreateBody { year: 2023 };
    // Act
    let response = app.create_year(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_returns_404_for_a_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_year(2020).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn get_returns_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_year(2022).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.year, 2022);
}

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn delete_returns_404_for_a_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.delete_year(2020).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("years", "months", "resources")
)]
async fn delete_returns_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.delete_year(2022).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.year, 2022);
}
