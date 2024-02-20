use datamize_domain::{ExpenseCategorization, ExpenseType, SubExpenseType, Uuid};
use fake::{Dummy, Fake, Faker};
use pretty_assertions::assert_eq;
use serde::Serialize;
use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test(migrations = "../db-postgres/migrations")]
async fn get_all_returns_empty_list_when_nothing_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_expenses_categorization().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("expenses_categorization")
)]
async fn get_all_returns_all_that_is_in_db(pool: PgPool) {
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_expenses_categorization().await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<ExpenseCategorization> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 4);
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct UpdateAllBody {
    id: Uuid,
    name: String,
    #[serde(rename = "type")]
    expense_type: ExpenseType,
    #[serde(rename = "sub_type")]
    sub_expense_type: SubExpenseType,
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("expenses_categorization")
)]
async fn update_all_adds_non_existing_categorization(pool: PgPool) {
    let app = spawn_app(pool).await;

    let body = vec![
        Faker.fake(),
        Faker.fake(),
        UpdateAllBody {
            id: "74a6048b-89fa-40d8-8946-6cea7bb170d3".parse().unwrap(),
            name: "Updated_Categorization".to_string(),
            expense_type: ExpenseType::Fixed,
            sub_expense_type: SubExpenseType::Housing,
        },
    ];

    // Act
    let response = app.update_all_expenses_categorization(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: Vec<ExpenseCategorization> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.len(), 3);
    assert_eq!(value[0].name, body[0].name);
    assert_eq!(value[1].name, body[1].name);
    assert_eq!(value[2].name, "Updated_Categorization");
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("expenses_categorization")
)]
async fn get_returns_404_for_a_non_existing_categorization(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .get_expense_categorization("1aa685cf-6cd3-4b53-94a9-a075886ec72b")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("expenses_categorization")
)]
async fn get_returns_existing_categorization(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .get_expense_categorization("74a6048b-89fa-40d8-8946-6cea7bb170d3")
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: ExpenseCategorization =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.name, "Expense_Cat_Test1");
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct UpdateBody {
    id: Uuid,
    name: String,
    #[serde(rename = "type")]
    expense_type: ExpenseType,
    #[serde(rename = "sub_type")]
    sub_expense_type: SubExpenseType,
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("expenses_categorization")
)]
async fn update_returns_404_for_a_non_existing_categorization(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    let body = UpdateBody {
        id: "2148461b-384b-43f6-916e-a15114f73a29".parse().unwrap(),
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_expense_categorization("2148461b-384b-43f6-916e-a15114f73a29", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test(
    migrations = "../db-postgres/migrations",
    fixtures("expenses_categorization")
)]
async fn update_changes_existing_categorization(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    let body = UpdateBody {
        id: "74a6048b-89fa-40d8-8946-6cea7bb170d3".parse().unwrap(),
        name: "Updated_name".to_string(),
        ..Faker.fake()
    };

    // Act
    let response = app
        .update_expense_categorization("74a6048b-89fa-40d8-8946-6cea7bb170d3", &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let value: ExpenseCategorization =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(value.name, "Updated_name");
}
