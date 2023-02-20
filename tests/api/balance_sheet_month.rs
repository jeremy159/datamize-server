use chrono::{Datelike, NaiveDate};
use datamize::domain::{Month, NetTotalType};
use fake::{faker::chrono::en::Date, Dummy, Fake, Faker};
use rand::Rng;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dummy_types::{
        DummyFinancialResource, DummyNetTotalType, DummyResourceCategory, DummyResourceType,
    },
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
async fn get_month_returns_a_400_for_invalid_year_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    app.insert_year(year).await;

    let min: i64 = i64::MAX - i32::MAX as i64;
    // Act
    let response = app
        .get_month(rand::thread_rng().gen_range(min..i64::MAX), month)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn get_month_returns_a_400_for_invalid_month_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;

    // Act
    let response = app.get_month(year, (13..i16::MAX).fake::<i16>()).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn get_month_returns_a_200_for_an_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;

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
    for nt in &month.net_totals {
        if nt.net_type == NetTotalType::Asset {
            assert_eq!(nt.id, month_net_total_assets.id);
            assert_eq!(nt.total, month_net_total_assets.total as i64);
        } else if nt.net_type == NetTotalType::Portfolio {
            assert_eq!(nt.id, month_net_total_portfolio.id);
            assert_eq!(nt.total, month_net_total_portfolio.total as i64);
        }
    }

    for r in &month.resources {
        if r.id == month_first_res.id {
            assert_eq!(r.balance, month_first_res.balance);
            assert_eq!(r.name, month_first_res.name);
        } else if r.id == month_second_res.id {
            assert_eq!(r.balance, month_second_res.balance);
            assert_eq!(r.name, month_second_res.name);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Body {
    resources: Vec<DummyFinancialResource>,
}

#[sqlx::test]
async fn put_month_returns_a_200_for_valid_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    let body = Body {
        resources: vec![Faker.fake(), Faker.fake()],
    };

    // Act
    let response = app.update_month(year, month, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[sqlx::test]
async fn put_month_returns_a_404_for_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let body = Body {
        resources: vec![Faker.fake()],
    };

    // Act
    let response = app.update_month(year, (1..12).fake::<i16>(), &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn put_month_returns_a_404_for_non_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    app.insert_year(year).await;
    let body = Body {
        resources: vec![Faker.fake()],
    };

    // Act
    let response = app.update_month(year, month, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn put_month_persists_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    let body = Body {
        resources: vec![Faker.fake()],
    };

    // Act
    app.update_month(year, month, &body).await;

    // Assert
    let saved = sqlx::query!("SELECT * FROM balance_sheet_resources",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch resources.");
    assert_eq!(saved.name, body.resources[0].name);
    assert_eq!(saved.balance, body.resources[0].balance);
    assert_eq!(saved.category, body.resources[0].category.to_string());
    assert_eq!(saved.r#type, body.resources[0].resource_type.to_string());
    assert_eq!(saved.editable, body.resources[0].editable);
}

#[sqlx::test]
async fn put_month_recompute_net_totals_with_previous_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month = (2..12).fake();
    let month1_id = app.insert_month(year_id, month).await;
    let month1_net_total_assets = app
        .insert_month_net_total(month1_id, DummyNetTotalType::Asset)
        .await;
    let month1_net_total_portfolio = app
        .insert_month_net_total(month1_id, DummyNetTotalType::Portfolio)
        .await;
    let prev_month = month - 1;
    let month2_id = app.insert_month(year_id, prev_month).await;
    let month2_net_total_assets = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Asset)
        .await;
    let month2_net_total_portfolio = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Portfolio)
        .await;

    let body = Body {
        resources: vec![DummyFinancialResource {
            category: DummyResourceCategory::Asset,
            resource_type: DummyResourceType::Cash,
            ..Faker.fake()
        }],
    };

    // Act
    let response = app.update_month(year, month, &body).await;
    let month: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for net in &month.net_totals {
        if net.net_type == NetTotalType::Asset {
            assert_eq!(net.total, body.resources[0].balance);
            assert_ne!(net.total, month1_net_total_assets.total as i64);
            assert_eq!(
                net.balance_var,
                body.resources[0].balance - month2_net_total_assets.total as i64
            );
        } else if net.net_type == NetTotalType::Portfolio {
            assert_eq!(net.total, body.resources[0].balance);
            assert_ne!(net.total, month1_net_total_portfolio.total as i64);
            assert_eq!(
                net.balance_var,
                body.resources[0].balance - month2_net_total_portfolio.total as i64
            );
        }
    }
}

#[sqlx::test]
async fn put_month_recompute_net_totals_with_previous_month_in_prev_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month = 1; // January
    let month1_id = app.insert_month(year_id, month).await;
    let prev_year = year - 1;
    let prev_year_id = app.insert_year(prev_year).await;
    let prev_month = 12; // December of prev year
    let month2_id = app.insert_month(prev_year_id, prev_month).await;
    let month1_net_total_assets = app
        .insert_month_net_total(month1_id, DummyNetTotalType::Asset)
        .await;
    let month1_net_total_portfolio = app
        .insert_month_net_total(month1_id, DummyNetTotalType::Portfolio)
        .await;
    let month2_net_total_assets = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Asset)
        .await;
    let month2_net_total_portfolio = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Portfolio)
        .await;

    let body = Body {
        resources: vec![DummyFinancialResource {
            category: DummyResourceCategory::Asset,
            resource_type: DummyResourceType::Cash,
            ..Faker.fake()
        }],
    };

    // Act
    let response = app.update_month(year, month, &body).await;
    let month: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for net in &month.net_totals {
        if net.net_type == NetTotalType::Asset {
            assert_eq!(net.total, body.resources[0].balance);
            assert_ne!(net.total, month1_net_total_assets.total as i64);
            assert_eq!(
                net.balance_var,
                body.resources[0].balance - month2_net_total_assets.total as i64
            );
        } else if net.net_type == NetTotalType::Portfolio {
            assert_eq!(net.total, body.resources[0].balance);
            assert_ne!(net.total, month1_net_total_portfolio.total as i64);
            assert_eq!(
                net.balance_var,
                body.resources[0].balance - month2_net_total_portfolio.total as i64
            );
        }
    }
}

#[sqlx::test]
async fn put_month_returns_a_422_for_wrong_root_body_attribute(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        res: Vec<DummyFinancialResource>,
    }
    let body = Body {
        res: vec![Faker.fake()],
    };

    // Act
    let response = app.update_month(year, month, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_month_returns_a_422_for_wrong_body_attributes(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct FinancialResourceWrongName {
        pub id: Uuid,
        pub name: String,
        pub category: DummyResourceCategory,
        #[serde(rename = "type")]
        pub resource_type: DummyResourceType,
        pub balanceeeeeeeee: i64,
        pub editable: bool,
    }
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        resources: Vec<FinancialResourceWrongName>,
    }
    let body = Body {
        resources: vec![Faker.fake()],
    };

    // Act
    let response = app.update_month(year, month, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_month_returns_a_422_for_missing_body_attributes(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct FinancialResourceMissing {
        pub id: Uuid,
        pub name: String,
        pub category: DummyResourceCategory,
        #[serde(rename = "type")]
        pub resource_type: DummyResourceType,
        // pub balance: i64,
        pub editable: bool,
    }
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        resources: Vec<FinancialResourceMissing>,
    }
    let body = Body {
        resources: vec![Faker.fake()],
    };

    // Act
    let response = app.update_month(year, month, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_month_returns_a_422_for_wrong_body_attribute_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct FinancialResourceWrongType {
        pub id: Uuid,
        pub name: i64,
        pub category: DummyResourceCategory,
        #[serde(rename = "type")]
        pub resource_type: DummyResourceType,
        pub balance: i64,
        pub editable: bool,
    }
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        resources: Vec<FinancialResourceWrongType>,
    }
    let body = Body {
        resources: vec![Faker.fake()],
    };

    // Act
    let response = app.update_month(year, month, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_month_returns_a_400_for_empty_body(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;

    // Act
    let response = app
        .api_client
        .put(&format!(
            "{}/api/balance_sheet/years/{}/months/{}",
            &app.address, year, month
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_month_returns_a_415_for_missing_content_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;

    // Act
    let response = app
        .api_client
        .put(&format!(
            "{}/api/balance_sheet/years/{}/months/{}",
            &app.address, year, month
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
async fn put_month_returns_a_400_for_invalid_year_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    let body = Body {
        resources: vec![Faker.fake()],
    };
    let min: i64 = i64::MAX - i32::MAX as i64;

    // Act
    let response = app
        .update_month(rand::thread_rng().gen_range(min..i64::MAX), month, &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_month_returns_a_400_for_invalid_month_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let month = date.month();
    let year_id = app.insert_year(year).await;
    app.insert_month(year_id, month as i16).await;
    let body = Body {
        resources: vec![Faker.fake()],
    };

    // Act
    let response = app
        .update_month(year, (13..i16::MAX).fake::<i16>(), &body)
        .await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}
