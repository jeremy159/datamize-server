use sqlx::PgPool;
use uuid::Uuid;

use crate::helpers::spawn_app;

#[sqlx::test]
async fn get_external_accounts_returns_a_500_if_database_returns_error(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Sabotage the database
    sqlx::query!("DROP TABLE external_accounts;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app
        .api_client
        .get(&format!(
            "{}/budget_providers/external/accounts",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn get_external_accounts_returns_a_200_if_database_returns_success(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO external_accounts (id, name, type, balance, username, encrypted_password, deleted) VALUES ($1, 'test', 'tfsa', 0, '', $2, false);",
        id, vec![0; 32]
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    // Act
    let response = app
        .api_client
        .get(&format!(
            "{}/budget_providers/external/accounts",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let accounts: Vec<datamize::web_scraper::account::ExternalAccount> =
        response.json().await.expect("Failed to parse JSON");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, id);
    assert_eq!(accounts[0].name, "test");
}
