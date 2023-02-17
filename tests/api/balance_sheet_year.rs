use chrono::{Datelike, NaiveDate};
use datamize::domain::{NetTotalType, YearDetail};
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
async fn get_year_returns_a_200_for_an_existing_year() {
    // Arange
    let app = spawn_app().await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_year(year).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
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

#[tokio::test]
async fn get_year_returns_net_totals_saving_rates_and_months_of_the_year() {
    // Arange
    let app = spawn_app().await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;

    let year_net_total_assets = app
        .insert_year_net_total(year_id, NetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, NetTotalType::Portfolio)
        .await;
    let saving_rate = app.insert_saving_rate(year_id).await;
    let month = app.insert_month(year_id).await;
    let month_net_total_assets = app
        .insert_month_net_total(month.0, NetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month.0, NetTotalType::Portfolio)
        .await;

    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for nt in &year.net_totals {
        if nt.net_type == NetTotalType::Asset {
            assert_eq!(nt.id, year_net_total_assets.0);
            assert_eq!(nt.total, month_net_total_assets.1);
        } else if nt.net_type == NetTotalType::Portfolio {
            assert_eq!(nt.id, year_net_total_portfolio.0);
            assert_eq!(nt.total, month_net_total_portfolio.1);
        }
    }

    // Make sure update on net_totals is persisted.
    let saved_net_total = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1 AND year_id = $2",
        year_net_total_assets.0,
        year.id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert_eq!(
        saved_net_total.total,
        year.net_totals
            .iter()
            .find(|nt| nt.net_type == NetTotalType::Asset)
            .unwrap()
            .total
    );

    for sr in &year.saving_rates {
        assert_eq!(sr.id, saving_rate.0);
    }

    for m in &year.months {
        assert_eq!(m.id, month.0);
    }
}

#[tokio::test]
async fn get_year_returns_net_totals_saving_rates_without_months_of_the_year() {
    // Arange
    let app = spawn_app().await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;

    let year_net_total_assets = app
        .insert_year_net_total(year_id, NetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, NetTotalType::Portfolio)
        .await;
    let saving_rate = app.insert_saving_rate(year_id).await;

    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for nt in &year.net_totals {
        if nt.net_type == NetTotalType::Asset {
            assert_eq!(nt.id, year_net_total_assets.0);
        } else if nt.net_type == NetTotalType::Portfolio {
            assert_eq!(nt.id, year_net_total_portfolio.0);
        }
    }

    for sr in &year.saving_rates {
        assert_eq!(sr.id, saving_rate.0);
    }

    assert!(year.months.is_empty());
}

#[tokio::test]
async fn get_year_returns_has_net_totals_update_persisted() {
    // Arange
    let app = spawn_app().await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;

    let year_net_total_assets = app
        .insert_year_net_total(year_id, NetTotalType::Asset)
        .await;
    let year_net_total_portfolio = app
        .insert_year_net_total(year_id, NetTotalType::Portfolio)
        .await;
    let month = app.insert_month(year_id).await;
    app.insert_month_net_total(month.0, NetTotalType::Asset)
        .await;
    app.insert_month_net_total(month.0, NetTotalType::Portfolio)
        .await;

    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    let saved_net_total_assets = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1 AND year_id = $2",
        year_net_total_assets.0,
        year.id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert_eq!(
        saved_net_total_assets.total,
        year.net_totals
            .iter()
            .find(|nt| nt.net_type == NetTotalType::Asset)
            .unwrap()
            .total
    );

    let saved_net_total_portfolio = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1 AND year_id = $2",
        year_net_total_portfolio.0,
        year.id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert_eq!(
        saved_net_total_portfolio.total,
        year.net_totals
            .iter()
            .find(|nt| nt.net_type == NetTotalType::Portfolio)
            .unwrap()
            .total
    );
}

// TODO: Test update of saving rates (PUT endpoint)
