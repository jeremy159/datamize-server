use chrono::{Datelike, NaiveDate};
use datamize_server::models::balance_sheet::Year;
use fake::{faker::chrono::en::Date, Fake};
use sqlx::PgPool;

use crate::{
    dummy_types::{DummyNetTotalType, DummyResourceCategory, DummyResourceType},
    helpers::spawn_app,
};

#[sqlx::test]
async fn get_year_returns_a_404_for_a_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_year(Date().fake::<NaiveDate>().year()).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn get_year_returns_a_200_for_an_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_year(year).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[sqlx::test]
async fn get_year_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
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

#[sqlx::test]
async fn get_year_returns_net_totals_saving_rates_months_and_resources_of_the_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;

    let year_net_total_assets = app
        .insert_year_net_total(year_id, DummyNetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, DummyNetTotalType::Portfolio)
        .await;
    let month = app.insert_random_month(year_id).await;
    let month_net_total_assets = app
        .insert_month_net_total(month.0, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month.0, DummyNetTotalType::Portfolio)
        .await;

    app.insert_financial_resource(
        month.0,
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;

    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(year.net_assets.id, year_net_total_assets.id);
    assert_eq!(year.net_assets.total, year_net_total_assets.total as i64);
    assert_eq!(year.net_portfolio.id, year_net_total_portfolio.id);
    assert_eq!(
        year.net_portfolio.total,
        year_net_total_portfolio.total as i64
    );

    assert_eq!(year.months.len(), 1);
    assert_eq!(year.months[0].id, month.0);
    assert_eq!(year.months[0].net_assets.id, month_net_total_assets.id);
    assert_eq!(
        year.months[0].net_assets.total,
        month_net_total_assets.total as i64
    );
    assert_eq!(
        year.months[0].net_portfolio.id,
        month_net_total_portfolio.id
    );
    assert_eq!(
        year.months[0].net_portfolio.total,
        month_net_total_portfolio.total as i64
    );
}

#[sqlx::test]
async fn get_year_returns_net_totals_saving_rates_without_months_of_the_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;

    let year_net_total_assets = app
        .insert_year_net_total(year_id, DummyNetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, DummyNetTotalType::Portfolio)
        .await;
    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(year.net_assets.id, year_net_total_assets.id);
    assert_eq!(year.net_portfolio.id, year_net_total_portfolio.id);

    assert!(year.months.is_empty());
}

#[sqlx::test]
async fn get_year_returns_has_net_totals_update_persisted(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;

    let year_net_total_assets = app
        .insert_year_net_total(year_id, DummyNetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, DummyNetTotalType::Portfolio)
        .await;
    let month = app.insert_random_month(year_id).await;
    app.insert_month_net_total(month.0, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month.0, DummyNetTotalType::Portfolio)
        .await;

    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    let saved_net_total_assets = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1 AND year_id = $2",
        year_net_total_assets.id,
        year.id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert_eq!(saved_net_total_assets.total, year.net_assets.total);

    let saved_net_total_portfolio = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1 AND year_id = $2",
        year_net_total_portfolio.id,
        year.id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert_eq!(saved_net_total_portfolio.total, year.net_portfolio.total);
}

#[sqlx::test]
async fn delete_year_returns_a_404_for_a_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();

    // Act
    let response = app.delete_year(year).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_year_returns_a_200_and_the_year_for_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let year_net_total_assets = app
        .insert_year_net_total(year_id, DummyNetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, DummyNetTotalType::Portfolio)
        .await;

    let month = date.month();
    let month_id = app.insert_month(year_id, month as i16).await;

    let month_net_total_assets = app
        .insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;

    app.insert_financial_resource(
        month_id,
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;

    // Act
    let response = app.delete_year(year).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let year: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(year.id, year_id);
    assert_eq!(year.net_assets.id, year_net_total_assets.id);
    assert_eq!(year.net_assets.total, year_net_total_assets.total as i64);
    assert_eq!(year.net_portfolio.id, year_net_total_portfolio.id);
    assert_eq!(
        year.net_portfolio.total,
        year_net_total_portfolio.total as i64
    );
    assert_eq!(year.months[0].id, month_id);
    assert_eq!(year.months[0].net_assets.id, month_net_total_assets.id);
    assert_eq!(
        year.months[0].net_portfolio.id,
        month_net_total_portfolio.id
    );
}

#[sqlx::test]
async fn delete_year_removes_year_month_saving_rates_and_net_totals_from_db_but_not_resource(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let year_net_total_assets = app
        .insert_year_net_total(year_id, DummyNetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, DummyNetTotalType::Portfolio)
        .await;

    let month = date.month();
    let month_id = app.insert_month(year_id, month as i16).await;

    let month_net_total_assets = app
        .insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;

    let res = app
        .insert_financial_resource(
            month_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    app.delete_year(year).await;

    // Assert
    let saved_year = sqlx::query!(
        "SELECT year FROM balance_sheet_years WHERE id = $1",
        year_id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved year.");
    assert!(saved_year.is_none());

    let saved_year_net_assets = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1",
        year_net_total_assets.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved net assets.");
    assert!(saved_year_net_assets.is_none());

    let saved_year_net_portfolio = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1",
        year_net_total_portfolio.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved net portfolio.");
    assert!(saved_year_net_portfolio.is_none());

    let saved_month = sqlx::query!(
        "SELECT month FROM balance_sheet_months WHERE id = $1",
        month_id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved month.");
    assert!(saved_month.is_none());

    let saved_month_net_assets = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE id = $1",
        month_net_total_assets.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved net assets.");
    assert!(saved_month_net_assets.is_none());

    let saved_month_net_portfolio = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE id = $1",
        month_net_total_portfolio.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved net portfolio.");
    assert!(saved_month_net_portfolio.is_none());

    let saved_resource = sqlx::query!(
        "SELECT * FROM balance_sheet_resources WHERE id = $1",
        res.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved resource.");
    assert!(saved_resource.is_some());
}
