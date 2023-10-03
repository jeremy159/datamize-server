use chrono::{Datelike, Months, NaiveDate};
use datamize_domain::{NetTotalType, Year};
use fake::faker::chrono::en::Date;
use fake::Fake;
use serde::Serialize;
use sqlx::PgPool;

use crate::dummy_types::DummyNetTotalType;
use crate::helpers::spawn_app;

#[sqlx::test]
async fn get_years_returns_an_empty_list_when_nothing_in_database(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_years_summary().await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_years_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    // Sabotage the database
    sqlx::query!("ALTER TABLE balance_sheet_years DROP COLUMN year;")
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.get_years_summary().await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn post_years_persists_the_new_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body {
        year: Date().fake::<NaiveDate>().year(),
    };

    // Act
    app.create_year(&body).await;

    // Assert
    let saved = sqlx::query!("SELECT year FROM balance_sheet_years",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved year.");
    assert_eq!(saved.year, body.year);
}

#[sqlx::test]
async fn post_years_returns_a_409_if_year_already_exists(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body { year };

    // Act
    let response = app.create_year(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[sqlx::test]
async fn post_years_persits_net_totals_and_saving_rates_for_new_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body {
        year: Date().fake::<NaiveDate>().year(),
    };

    // Act
    let response = app.create_year(&body).await;
    let year: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    let saved_net_totals = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE year_id = $1",
        year.id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert!(!saved_net_totals.is_empty());
}

#[sqlx::test]
async fn post_years_updates_net_totals_if_previous_year_exists(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year_id = app
        .insert_year(date.checked_sub_months(Months::new(12)).unwrap().year())
        .await;

    let net_total_assets = app
        .insert_year_net_total(year_id, DummyNetTotalType::Asset)
        .await;

    let net_total_portfolio = app
        .insert_year_net_total(year_id, DummyNetTotalType::Portfolio)
        .await;

    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body { year: date.year() };

    // Act
    let response = app.create_year(&body).await;
    let year: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(year.net_assets.balance_var, -net_total_assets.total as i64);
    assert_eq!(year.net_assets.percent_var, -1.0);
    assert_eq!(
        year.net_portfolio.balance_var,
        -net_total_portfolio.total as i64
    );
    assert_eq!(year.net_portfolio.percent_var, -1.0);
}

#[sqlx::test]
async fn post_years_updates_net_totals_of_current_and_next_year_if_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let prev_year_id = app
        .insert_year(date.checked_sub_months(Months::new(12)).unwrap().year())
        .await;

    let prev_net_total_assets = app
        .insert_year_net_total(prev_year_id, DummyNetTotalType::Asset)
        .await;

    let prev_net_total_portfolio = app
        .insert_year_net_total(prev_year_id, DummyNetTotalType::Portfolio)
        .await;

    let next_year_id = app
        .insert_year(date.checked_add_months(Months::new(12)).unwrap().year())
        .await;

    let next_net_total_assets = app
        .insert_year_net_total(next_year_id, DummyNetTotalType::Asset)
        .await;

    let next_net_total_portfolio = app
        .insert_year_net_total(next_year_id, DummyNetTotalType::Portfolio)
        .await;

    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body { year: date.year() };

    // Act
    let response = app.create_year(&body).await;
    let year: Year = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(
        year.net_assets.balance_var,
        -prev_net_total_assets.total as i64
    );
    assert_eq!(year.net_assets.percent_var, -1.0);
    assert_eq!(
        year.net_portfolio.balance_var,
        -prev_net_total_portfolio.total as i64
    );
    assert_eq!(year.net_portfolio.percent_var, -1.0);

    // Get net totals of next year
    let saved_next_net_totals = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE year_id = $1",
        next_year_id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");

    for next_nt in saved_next_net_totals {
        if next_nt.r#type == NetTotalType::Asset.to_string() {
            assert_ne!(next_nt.balance_var, next_net_total_assets.balance_var);
            assert_ne!(next_nt.percent_var, next_net_total_assets.percent_var);
        } else {
            assert_ne!(next_nt.balance_var, next_net_total_portfolio.balance_var);
            assert_ne!(next_nt.percent_var, next_net_total_portfolio.percent_var);
        }
    }
}
