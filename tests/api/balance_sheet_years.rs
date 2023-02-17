use chrono::{Datelike, Months, NaiveDate};
use datamize::domain::{NetTotalType, YearDetail};
use fake::faker::chrono::en::Date;
use fake::Fake;
use serde::Serialize;

use crate::helpers::spawn_app;

#[tokio::test]
async fn get_years_returns_an_empty_list_when_nothing_in_database() {
    // Arange
    let app = spawn_app().await;

    // Act
    let response = app.get_years_summary().await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[tokio::test]
async fn get_years_fails_if_there_is_a_fatal_database_error() {
    // Arange
    let app = spawn_app().await;
    // Sabotage the database
    sqlx::query!("ALTER TABLE balance_sheet_years DROP COLUMN year;",)
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

#[tokio::test]
async fn post_years_returns_a_201_for_valid_body_data() {
    // Arange
    let app = spawn_app().await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body {
        year: Date().fake::<NaiveDate>().year(),
    };

    // Act
    let response = app.create_year(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[tokio::test]
async fn post_years_returns_a_422_for_invalid_body_format_data() {
    // Arange
    let app = spawn_app().await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: String,
    }
    let body = Body {
        year: Date().fake::<NaiveDate>().year().to_string(),
    };

    // Act
    let response = app.create_year(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn post_years_returns_a_400_for_empty_body() {
    // Arange
    let app = spawn_app().await;

    // Act
    let response = app
        .api_client
        .post(&format!("{}/api/balance_sheet/years", &app.address))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn post_years_returns_a_415_for_missing_json_content_type() {
    // Arange
    let app = spawn_app().await;

    // Act
    let response = app
        .api_client
        .post(&format!("{}/api/balance_sheet/years", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
}

#[tokio::test]
async fn post_years_persists_the_new_year() {
    // Arange
    let app = spawn_app().await;
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

#[tokio::test]
async fn post_years_returns_a_409_if_year_already_exists() {
    // Arange
    let app = spawn_app().await;
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

#[tokio::test]
async fn post_years_persits_net_totals_and_saving_rates_for_new_year() {
    // Arange
    let app = spawn_app().await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body {
        year: Date().fake::<NaiveDate>().year(),
    };

    // Act
    let response = app.create_year(&body).await;
    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    let saved_net_totals = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE year_id = $1",
        year.id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert!(!saved_net_totals.is_empty());

    let saved_saving_rates = sqlx::query!(
        "SELECT * FROM balance_sheet_saving_rates WHERE year_id = $1",
        year.id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch saving rates.");
    assert!(!saved_saving_rates.is_empty());
}

#[tokio::test]
async fn post_years_updates_net_totals_if_previous_year_exists() {
    // Arange
    let app = spawn_app().await;
    let date = Date().fake::<NaiveDate>();
    let year_id = app
        .insert_year(date.checked_sub_months(Months::new(12)).unwrap().year())
        .await;

    let (_, total_assets, _, _) = app
        .insert_year_net_total(year_id, NetTotalType::Asset)
        .await;

    let (_, total_portfolio, _, _) = app
        .insert_year_net_total(year_id, NetTotalType::Portfolio)
        .await;

    #[derive(Debug, Clone, Serialize)]
    struct Body {
        year: i32,
    }
    let body = Body { year: date.year() };

    // Act
    let response = app.create_year(&body).await;
    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for net in &year.net_totals {
        if net.net_type == NetTotalType::Asset {
            assert_eq!(net.balance_var, -total_assets);
        } else if net.net_type == NetTotalType::Portfolio {
            assert_eq!(net.balance_var, -total_portfolio);
        }
        assert_eq!(net.percent_var, -1.0);
    }
}
