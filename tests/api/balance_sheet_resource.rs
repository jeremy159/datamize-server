use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use datamize::domain::FinancialResourceYearly;
use fake::{faker::chrono::en::Date, Dummy, Fake, Faker};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    dummy_types::{DummyMonthNum, DummyNetTotalType, DummyResourceCategory, DummyResourceType},
    helpers::spawn_app,
};

#[sqlx::test]
async fn get_resource_returns_a_404_for_a_non_existing_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_resource(Faker.fake::<Uuid>()).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn get_resource_returns_a_400_for_invalid_resource_id_format_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_resource(Faker.fake::<i64>()).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn get_resource_returns_a_200_for_an_existing_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = app.insert_random_month(year_id).await;
    let res = app
        .insert_financial_resource(
            month.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    let response = app.get_resource(res.id).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[sqlx::test]
async fn get_resource_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = app.insert_random_month(year_id).await;
    let res = app
        .insert_financial_resource(
            month.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    // Sabotage the database
    sqlx::query!("ALTER TABLE balance_sheet_resources_months DROP COLUMN balance;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.get_resource(res.id).await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn get_resource_returns_all_months_with_balance_of_the_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month1 = (1..4).fake::<i16>();
    let month1_id = app.insert_month(year_id, month1).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month2 = (5..9).fake::<i16>();
    let month2_id = app.insert_month(year_id, month2).await;
    app.insert_financial_resource_with_id_in_month(month2_id, res.id)
        .await;
    // Month without any balance
    let month3 = (10..12).fake::<i16>();
    app.insert_month(year_id, month3).await;

    // Act
    let response = app.get_resource(res.id).await;
    assert!(response.status().is_success());

    let resource: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert!(resource
        .balance_per_month
        .get(&(month1.try_into().unwrap()))
        .is_some());
    assert!(resource
        .balance_per_month
        .get(&(month2.try_into().unwrap()))
        .is_some());
    assert!(resource
        .balance_per_month
        .get(&(month3.try_into().unwrap()))
        .is_none());
}

// #[sqlx::test]
// TODO: To tackle later. Maybe a resource should contain a reference to multiple years (including multiple months)?
async fn get_resource_returns_months_with_balance_only_of_requested_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month1 = (1..4).fake::<i16>();
    let month1_id = app.insert_month(year_id, month1).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month2 = (5..9).fake::<i16>();
    let month2_id = app.insert_month(year_id, month2).await;
    let month2_balance = app
        .insert_financial_resource_with_id_in_month(month2_id, res.id)
        .await;
    // Same month but different year
    let next_year = year + 1;
    let next_year_id = app.insert_year(next_year).await;
    let month2_of_next_year_id = app.insert_month(next_year_id, month2).await;
    let month2_of_next_year_balance = app
        .insert_financial_resource_with_id_in_month(month2_of_next_year_id, res.id)
        .await;

    // Act
    let response = app.get_resource(res.id).await;
    assert!(response.status().is_success());

    let resource: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert!(resource
        .balance_per_month
        .get(&(month1.try_into().unwrap()))
        .is_some());
    let month2_received_balance = resource
        .balance_per_month
        .get(&(month2.try_into().unwrap()));
    assert!(month2_received_balance.is_some());
    assert_eq!(*month2_received_balance.unwrap(), month2_balance);
    assert_ne!(
        *month2_received_balance.unwrap(),
        month2_of_next_year_balance
    );
}

#[derive(Debug, Clone, Serialize, Dummy)]
struct Body {
    name: String,
    category: DummyResourceCategory,
    #[serde(rename = "type")]
    r_type: DummyResourceType,
    editable: bool,
    year: i32,
    balance_per_month: BTreeMap<DummyMonthNum, i64>,
}

#[sqlx::test]
async fn put_resource_returns_a_200_for_valid_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month1, Faker.fake::<i64>());
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    let response = app.update_resource(res.id, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[sqlx::test]
async fn put_resource_returns_a_404_for_non_existing_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let body: Body = Faker.fake();

    // Act
    let response = app.update_resource(Faker.fake::<Uuid>(), &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn put_resource_returns_a_404_for_non_existing_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let body: Body = Faker.fake();

    // Act
    let response = app.update_resource(res.id, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn put_resource_returns_a_404_for_non_existing_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    app.insert_month(year_id, month1 as i16).await;
    let body: Body = Faker.fake();

    // Act
    let response = app.update_resource(Faker.fake::<Uuid>(), &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn put_resource_persists_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let mut balance_per_month = BTreeMap::new();
    let new_balance: i64 = Faker.fake();
    balance_per_month.insert(month1, new_balance);
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    app.update_resource(res.id, &body).await;

    // Assert
    let saved = sqlx::query!("SELECT * FROM balance_sheet_resources",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved resource.");
    assert_eq!(saved.name, body.name);
    assert_eq!(saved.category, body.category.to_string());
    assert_eq!(saved.r#type, body.r_type.to_string());

    let saved_balance = sqlx::query!("SELECT balance FROM balance_sheet_resources_months WHERE resource_id = $1 AND month_id = $2", res.id, month1_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved balance.");

    assert_eq!(saved_balance.balance, new_balance);
}

#[sqlx::test]
async fn put_resource_returns_a_422_for_wrong_body_attribute(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month1, Faker.fake::<i64>());
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct Body {
        name: String,
        category: DummyResourceCategory,
        #[serde(rename = "type")]
        r_type: DummyResourceType,
        editableeeeeeeeeeeeeeeeeeeee: bool,
        year: i32,
        balance_per_month: BTreeMap<DummyMonthNum, i64>,
    }
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    let response = app.update_resource(res.id, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_resource_returns_a_422_for_missing_body_attributes(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month1, Faker.fake::<i64>());
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct Body {
        name: String,
        category: DummyResourceCategory,
        #[serde(rename = "type")]
        r_type: DummyResourceType,
        // editable: bool,
        year: i32,
        balance_per_month: BTreeMap<DummyMonthNum, i64>,
    }
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    let response = app.update_resource(res.id, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_resource_returns_a_422_for_wrong_body_attribute_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month1, Faker.fake::<i64>());
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct Body {
        name: i64,
        category: DummyResourceCategory,
        #[serde(rename = "type")]
        r_type: DummyResourceType,
        editable: bool,
        year: i32,
        balance_per_month: BTreeMap<DummyMonthNum, i64>,
    }
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    let response = app.update_resource(res.id, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn put_resource_returns_a_400_for_empty_body(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    let response = app
        .api_client
        .put(&format!(
            "{}/api/balance_sheet/resources/{}",
            &app.address, res.id
        ))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_resource_returns_a_415_for_missing_content_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    let response = app
        .api_client
        .put(&format!(
            "{}/api/balance_sheet/resources/{}",
            &app.address, res.id
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
async fn put_resource_returns_a_400_for_invalid_res_id_in_path(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    app.insert_financial_resource(
        month1_id,
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month1, Faker.fake::<i64>());
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    let response = app.update_resource(Faker.fake::<i64>(), &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_resource_persits_net_totals_for_month_and_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let mut balance_per_month = BTreeMap::new();
    let new_balance: i64 = Faker.fake();
    balance_per_month.insert(month1, new_balance);
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    app.update_resource(res.id, &body).await;

    // Assert
    let saved_month_net_totals = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE month_id = $1",
        month1_id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert!(!saved_month_net_totals.is_empty());

    let saved_year_net_totals = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_years WHERE year_id = $1",
        year_id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert!(!saved_year_net_totals.is_empty());
}

#[sqlx::test]
async fn put_resource_updates_net_totals_if_previous_month_exists(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month = (2..12).fake();
    let month_id = app.insert_month(year_id, month).await;
    // prev month
    let prev_month = month - 1;
    let month2_id = app.insert_month(year_id, prev_month).await;
    let month_net_total_assets = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Portfolio)
        .await;
    // res
    let res = app
        .insert_financial_resource(
            month_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let mut balance_per_month = BTreeMap::new();
    let month_balance = Faker.fake::<i32>() as i64;
    balance_per_month.insert(month.try_into().unwrap(), month_balance);
    let body = Body {
        year,
        balance_per_month,
        category: DummyResourceCategory::Asset,
        r_type: DummyResourceType::Cash,
        ..Faker.fake()
    };

    // Act
    app.update_resource(res.id, &body).await;

    // Assert
    let saved_month_net_assets = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE month_id = $1 AND type = 'asset'",
        month_id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    let saved_month_net_portfolio = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE month_id = $1 AND type = 'portfolio'",
        month_id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");

    assert_eq!(
        saved_month_net_assets[0].balance_var,
        month_balance - month_net_total_assets.total as i64
    );
    assert_eq!(
        saved_month_net_portfolio[0].balance_var,
        month_balance - month_net_total_portfolio.total as i64
    );
}

#[sqlx::test]
async fn delete_resource_returns_a_404_for_a_non_existing_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    app.insert_financial_resource(
        month1_id,
        DummyResourceCategory::Asset,
        DummyResourceType::Cash,
    )
    .await;

    // Act
    let response = app.delete_resource(Faker.fake::<Uuid>()).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_resource_returns_a_200_and_the_resource_for_existing_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month2: DummyMonthNum = month1.pred();
    let month2_id = app.insert_month(year_id, month2 as i16).await;
    let other_month_balance = app
        .insert_financial_resource_with_id_in_month(month2_id, res.id)
        .await;

    // Act
    let response = app.delete_resource(res.id).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let resource: FinancialResourceYearly =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(resource.base.id, res.id);
    assert_eq!(resource.base.category.to_string(), res.category.to_string());
    assert_eq!(resource.base.editable, res.editable);
    assert_eq!(resource.base.name, res.name);
    assert_eq!(
        resource.base.r_type.to_string(),
        res.resource_type.to_string()
    );
    assert_eq!(resource.year, year);
    assert_eq!(resource.balance_per_month.len(), 2);
    assert_eq!(
        *(resource
            .balance_per_month
            .get(&(month1 as i16).try_into().unwrap())
            .unwrap()),
        res.balance
    );
    assert_eq!(
        *(resource
            .balance_per_month
            .get(&(month2 as i16).try_into().unwrap())
            .unwrap()),
        other_month_balance
    );
}

#[sqlx::test]
async fn delete_resource_removes_resource_and_balance_associated_with_it(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let date = Date().fake::<NaiveDate>();
    let year = date.year();
    let year_id = app.insert_year(year).await;
    let month1: DummyMonthNum = date.month().try_into().unwrap();
    let month1_id = app.insert_month(year_id, month1 as i16).await;
    let res = app
        .insert_financial_resource(
            month1_id,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    app.delete_resource(res.id).await;

    // Assert
    let saved_res = sqlx::query!(
        "SELECT * FROM balance_sheet_resources WHERE id = $1",
        res.id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved resource.");
    assert!(saved_res.is_none());

    let saved_balance = sqlx::query!(
        "SELECT * FROM balance_sheet_resources_months WHERE resource_id = $1 AND month_id = $2",
        res.id,
        month1_id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved balance.");
    assert!(saved_balance.is_none());

    let saved_month = sqlx::query!(
        "SELECT * FROM balance_sheet_months WHERE id = $1",
        month1_id
    )
    .fetch_optional(&app.db_pool)
    .await
    .expect("Failed to fetch saved month.");
    assert!(saved_month.is_some());
}
