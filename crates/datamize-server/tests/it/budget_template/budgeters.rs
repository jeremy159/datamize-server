use datamize_domain::{BudgeterConfig, Uuid};
use fake::{Dummy, Fake, Faker};
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_budgeters().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn get_all_returns_all_that_is_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_budgeters().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<BudgeterConfig> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 3);
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct CreateBody {
    name: String,
    payee_ids: Vec<Uuid>,
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn create_returns_201_for_valid_body(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body: CreateBody = Faker.fake();
    // Act
    let response = app.create_budgeter(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn create_returns_409_for_budgeter_already_present(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body = CreateBody {
        name: "Budgeter_Test1".to_string(),
        ..Faker.fake()
    };
    // Act
    let response = app.create_budgeter(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn get_returns_404_for_a_non_existing_budgeter(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .get_budgeter("0b529b37-c405-4656-8fcf-1d54b93a3911")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn get_returns_existing_budgeter(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .get_budgeter("e399da25-8807-4f5b-850b-6c70b66f529c")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: BudgeterConfig = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.name, "Budgeter_Test1");
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct UpdateBody {
    id: Uuid,
    name: String,
    payee_ids: Vec<Uuid>,
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn update_returns_404_for_a_non_existing_budgeter(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    let body = UpdateBody {
        id: "8655bbdf-2cf0-4603-9e67-fbf7f0aa83d0".parse().unwrap(),
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_budgeter("8655bbdf-2cf0-4603-9e67-fbf7f0aa83d0", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn update_changes_existing_budgeter(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    let body = UpdateBody {
        id: "e399da25-8807-4f5b-850b-6c70b66f529c".parse().unwrap(),
        name: "Updated_name".to_string(),
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_budgeter("e399da25-8807-4f5b-850b-6c70b66f529c", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: BudgeterConfig = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.name, "Updated_name");
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn delete_returns_404_for_a_non_existing_budgeter(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .delete_budgeter("23b35048-c680-4d39-8704-c3c290e366f6")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../db-postgres/migrations", fixtures("budgeters"))]
async fn delete_returns_existing_budgeter(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .delete_budgeter("3b162522-e282-4e15-8da3-797e18d47f8d")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: BudgeterConfig = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.name, "Budgeter_Test2");
}
