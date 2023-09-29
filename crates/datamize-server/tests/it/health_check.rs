use sqlx::PgPool;

use crate::helpers::spawn_app;

#[sqlx::test]
async fn health_check_works(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}