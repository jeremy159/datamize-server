use chrono::{Datelike, NaiveDate};
use datamize_domain::Month;
use fake::{faker::chrono::en::Date, Fake};
use sqlx::PgPool;

use crate::{
    dummy_types::{DummyNetTotalType, DummyResourceCategory, DummyResourceType},
    helpers::spawn_app,
};

#[sqlx::test]
async fn get_month_returns_a_404_for_a_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();

    // Act
    let response = app.get_month(year, month).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn get_month_returns_a_404_for_a_non_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    app.insert_year(year).await;
    let month = date.month();

    // Act
    let response = app.get_month(year, month).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn get_month_returns_a_200_for_an_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    let month_id = app.insert_month(year_id, month as i16).await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;

    // Act
    let response = app.get_month(year, month).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[sqlx::test]
async fn get_month_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    // Sabotage the database
    sqlx::query!("ALTER TABLE balance_sheet_net_totals_months DROP COLUMN total;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.get_month(year, month).await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn get_month_returns_net_totals_and_resources_of_the_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    let month_id = app.insert_month(year_id, month as i16).await;

    let month_net_total_assets = app
        .insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;
    let month_first_res = app
        .insert_financial_resource(
            month_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month_second_res = app
        .insert_financial_resource(
            month_id,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    let response = app.get_month(year, month).await;
    assert!(response.status().is_success());

    let month: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(month.net_assets.id, month_net_total_assets.id);
    assert_eq!(month.net_assets.total, month_net_total_assets.total as i64);
    assert_eq!(month.net_portfolio.id, month_net_total_portfolio.id);
    assert_eq!(
        month.net_portfolio.total,
        month_net_total_portfolio.total as i64
    );

    for r in &month.resources {
        if r.base.id == month_first_res.id {
            assert_eq!(r.balance, month_first_res.balance);
            assert_eq!(r.base.name, month_first_res.name);
        } else if r.base.id == month_second_res.id {
            assert_eq!(r.balance, month_second_res.balance);
            assert_eq!(r.base.name, month_second_res.name);
        }
    }
}

#[sqlx::test]
async fn delete_month_returns_a_404_for_a_non_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    app.insert_year(year).await;
    let month = date.month();

    // Act
    let response = app.delete_month(year, month).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_month_returns_a_200_and_the_month_for_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month = date.month();
    let month_id = app.insert_month(year_id, month as i16).await;

    let month_net_total_assets = app
        .insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;

    // Act
    let response = app.delete_month(year, month).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let month: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(month.id, month_id);
    assert_eq!(month.net_assets.id, month_net_total_assets.id);
    assert_eq!(month.net_assets.total, month_net_total_assets.total as i64);
    assert_eq!(month.net_portfolio.id, month_net_total_portfolio.id);
    assert_eq!(
        month.net_portfolio.total,
        month_net_total_portfolio.total as i64
    );
}

#[sqlx::test]
async fn delete_month_removes_month_and_net_totals_from_db(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month = date.month();
    let month_id = app.insert_month(year_id, month as i16).await;

    let month_net_total_assets = app
        .insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;

    // Act
    app.delete_month(year, month).await;

    // Assert
    let saved_month = sqlx::query!(
        "SELECT month FROM balance_sheet_months WHERE id = $1",
        month_id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved month.");
    assert!(saved_month.is_none());

    let saved_net_assets = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE id = $1",
        month_net_total_assets.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved net assets.");
    assert!(saved_net_assets.is_none());

    let saved_net_portfolio = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE id = $1",
        month_net_total_portfolio.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved net portfolio.");
    assert!(saved_net_portfolio.is_none());
}

#[sqlx::test]
async fn delete_month_does_not_delete_same_month_of_different_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month = date.month();
    let month_id = app.insert_month(year_id, month as i16).await;
    let next_year_id = app.insert_year(year + 1).await;
    let same_month_id = app.insert_month(next_year_id, month as i16).await;

    app.insert_month_net_total(month_id, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month_id, DummyNetTotalType::Portfolio)
        .await;

    // Act
    app.delete_month(year, month).await;

    // Assert
    let saved_month = sqlx::query!(
        "SELECT month FROM balance_sheet_months WHERE id = $1",
        month_id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved month.");
    assert!(saved_month.is_none());

    let saved_same_month = sqlx::query!(
        "SELECT * FROM balance_sheet_months WHERE id = $1",
        same_month_id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved month.");
    assert!(saved_same_month.is_some());
    assert_eq!(saved_same_month.unwrap().id, same_month_id);
}
