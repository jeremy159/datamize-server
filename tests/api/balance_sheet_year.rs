use chrono::{Datelike, NaiveDate};
use fake::{faker::chrono::en::Date, Fake};

use crate::helpers::spawn_app;

#[tokio::test]
async fn get_year_returns_a_404_for_a_non_existing_year() {
    // Arange
    let app = spawn_app().await;

    // Act
    let response = app.get_year(Date().fake::<NaiveDate>().year()).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_year_fails_if_there_is_a_fatal_database_error() {
    // Arange
    let app = spawn_app().await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    // Sabotage the database
    sqlx::query!("ALTER TABLE balance_sheet_net_totals_years DROP COLUMN total;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.get_year(year).await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

// TODO: Also test when year doesn't have month - it should not fail.
// TODO: Also test, year with last month that needs update gets the update persisted in db.
// TODO: Test update of saving rates (PUT endpoint)
// #[tokio::test]
async fn get_year_returns_net_totals_saving_rates_and_months_of_the_year() {
    // Arange
    let app = spawn_app().await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;

    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, year_id)
        VALUES ($1, 'asset', 20000, 0.0, 0.0, $2);
        "#,
        uuid::Uuid::new_v4(),
        year_id,
    )
    .execute(&app.db_pool)
    .await
    .expect("Failed to insert net totals of a year.");
    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, year_id)
        VALUES ($1, 'portfolio', 1000, 0.0, 0.0, $2);
        "#,
        uuid::Uuid::new_v4(),
        year_id,
    )
    .execute(&app.db_pool)
    .await
    .expect("Failed to insert net totals of a year.");
    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_saving_rates (id, name, savings, employer_contribution, employee_contribution, mortgage_capital, incomes, rate, year_id)
        VALUES ($1, 'test', 1000, 1500, 1500, 2000, 10000, 0.25, $2);
        "#,
        uuid::Uuid::new_v4(),
        year_id,
    )
    .execute(&app.db_pool)
    .await
    .expect("Failed to insert saving rates of a year.");

    // Act
    let response = app.get_year(2023).await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}
