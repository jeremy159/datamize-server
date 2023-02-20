use chrono::{Datelike, NaiveDate};
use datamize::domain::{NetTotalType, YearDetail};
use fake::{faker::chrono::en::Date, Dummy, Fake, Faker};
use rand::Rng;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dummy_types::{DummyNetTotalType, DummySavingRatesPerPerson},
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
async fn get_year_returns_a_400_for_invalid_year_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    let min: i64 = i64::MAX - i32::MAX as i64;
    // Act
    let response = app
        .get_year(rand::thread_rng().gen_range(min..i64::MAX))
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
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
async fn get_year_returns_net_totals_saving_rates_and_months_of_the_year(pool: PgPool) {
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
    let saving_rate = app.insert_saving_rate(year_id).await;
    let month = app.insert_random_month(year_id).await;
    let month_net_total_assets = app
        .insert_month_net_total(month.0, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month.0, DummyNetTotalType::Portfolio)
        .await;

    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for nt in &year.net_totals {
        if nt.net_type == NetTotalType::Asset {
            assert_eq!(nt.id, year_net_total_assets.id);
            assert_eq!(nt.total, month_net_total_assets.total as i64);
        } else if nt.net_type == NetTotalType::Portfolio {
            assert_eq!(nt.id, year_net_total_portfolio.id);
            assert_eq!(nt.total, month_net_total_portfolio.total as i64);
        }
    }

    // Make sure update on net_totals is persisted.
    let saved_net_total = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1 AND year_id = $2",
        year_net_total_assets.id,
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
        assert_eq!(sr.id, saving_rate.id);
    }

    for m in &year.months {
        assert_eq!(m.id, month.0);
    }
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
    let saving_rate = app.insert_saving_rate(year_id).await;

    // Act
    let response = app.get_year(year).await;
    assert!(response.status().is_success());

    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for nt in &year.net_totals {
        if nt.net_type == NetTotalType::Asset {
            assert_eq!(nt.id, year_net_total_assets.id);
        } else if nt.net_type == NetTotalType::Portfolio {
            assert_eq!(nt.id, year_net_total_portfolio.id);
        }
    }

    for sr in &year.saving_rates {
        assert_eq!(sr.id, saving_rate.id);
    }

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

    let year: YearDetail = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    let saved_net_total_assets = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE id = $1 AND year_id = $2",
        year_net_total_assets.id,
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
        year_net_total_portfolio.id,
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

#[derive(Debug, Clone, Serialize)]
struct Body {
    saving_rates: Vec<DummySavingRatesPerPerson>,
}

#[sqlx::test]
async fn put_year_returns_a_200_for_valid_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    let body = Body {
        saving_rates: vec![Faker.fake(), Faker.fake()],
    };

    // Act
    let response = app.update_year(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[sqlx::test]
async fn put_year_returns_a_404_for_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let body = Body {
        saving_rates: vec![Faker.fake()],
    };

    // Act
    let response = app.update_year(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn put_year_persists_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    let body = Body {
        saving_rates: vec![Faker.fake()],
    };

    // Act
    app.update_year(year, &body).await;

    // Assert
    let saved = sqlx::query!("SELECT * FROM balance_sheet_saving_rates",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved saving rate.");
    assert_eq!(saved.name, body.saving_rates[0].name);
    assert_eq!(saved.savings, body.saving_rates[0].savings);
    assert_eq!(
        saved.employer_contribution,
        body.saving_rates[0].employer_contribution
    );
    assert_eq!(
        saved.employee_contribution,
        body.saving_rates[0].employee_contribution
    );
    assert_eq!(
        saved.mortgage_capital,
        body.saving_rates[0].mortgage_capital
    );
    assert_eq!(saved.incomes, body.saving_rates[0].incomes);
    assert_eq!(saved.rate, body.saving_rates[0].rate);
}

#[sqlx::test]
async fn put_year_returns_a_422_for_wrong_root_body_attribute(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        savings: Vec<DummySavingRatesPerPerson>,
    }
    let body = Body {
        savings: vec![Faker.fake()],
    };

    // Act
    let response = app.update_year(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_year_returns_a_422_for_wrong_body_attributes(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct SavingRatesPerPersonWrongName {
        pub id: Uuid,
        pub name: String,
        pub savings: i64,
        pub employer_contribution: i64,
        pub employeeeeeeeeee_contribution: i64,
        pub mortgage_capital: i64,
        pub incomes: i64,
        pub rate: f32,
    }
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        saving_rates: Vec<SavingRatesPerPersonWrongName>,
    }
    let body = Body {
        saving_rates: vec![Faker.fake()],
    };

    // Act
    let response = app.update_year(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_year_returns_a_422_for_missing_body_attributes(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct SavingRatesPerPersonMissing {
        pub id: Uuid,
        pub name: String,
        pub savings: i64,
        pub employer_contribution: i64,
        // pub employee_contribution: i64,
        pub mortgage_capital: i64,
        pub incomes: i64,
        pub rate: f32,
    }
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        saving_rates: Vec<SavingRatesPerPersonMissing>,
    }
    let body = Body {
        saving_rates: vec![Faker.fake()],
    };

    // Act
    let response = app.update_year(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_year_returns_a_422_for_wrong_body_attribute_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct SavingRatesPerPersonMissing {
        pub id: Uuid,
        pub name: i64,
        pub savings: i64,
        pub employer_contribution: i64,
        pub employee_contribution: i64,
        pub mortgage_capital: i64,
        pub incomes: i64,
        pub rate: f32,
    }
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        saving_rates: Vec<SavingRatesPerPersonMissing>,
    }
    let body = Body {
        saving_rates: vec![Faker.fake()],
    };

    // Act
    let response = app.update_year(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_year_returns_a_400_for_empty_body(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app
        .api_client
        .put(&format!(
            "{}/api/balance_sheet/years/{}",
            &app.address, year
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_year_returns_a_415_for_missing_content_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app
        .api_client
        .put(&format!(
            "{}/api/balance_sheet/years/{}",
            &app.address, year
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
}

#[sqlx::test]
async fn put_year_returns_a_400_for_invalid_year_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    let body = Body {
        saving_rates: vec![Faker.fake()],
    };

    let min: i64 = i64::MAX - i32::MAX as i64;
    // Act
    let response = app
        .update_year(rand::thread_rng().gen_range(min..i64::MAX), &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}
