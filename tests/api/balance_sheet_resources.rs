use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use datamize::domain::FinancialResourceYearly;
use fake::faker::chrono::en::Date;
use fake::{Dummy, Fake, Faker};
use serde::Serialize;
use sqlx::PgPool;

use crate::dummy_types::{
    DummyMonthNum, DummyNetTotalType, DummyResourceCategory, DummyResourceType,
};
use crate::helpers::spawn_app;

#[sqlx::test]
async fn get_all_resources_returns_an_empty_list_when_nothing_in_database(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_resources().await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_all_resources_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    // Sabotage the database
    sqlx::query!("ALTER TABLE balance_sheet_years DROP COLUMN year;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.get_all_resources().await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn get_all_resources_returns_an_empty_list_even_if_year_is_in_db(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_all_resources().await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_all_resources_returns_all_resources_of_only_years_with_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let next_year = year + 1;
    app.insert_year(next_year).await;
    let month1 = app.insert_random_month(year_id).await;
    let res1_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    let month2 = app.insert_random_month(year_id).await;
    let res1_month2 = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month2 = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Liability,
            DummyResourceType::LongTerm,
        )
        .await;

    let all_res = [
        res1_month1.id,
        res2_month1.id,
        res1_month2.id,
        res2_month2.id,
    ];

    // Act
    let response = app.get_all_resources().await;
    assert!(response.status().is_success());

    let resources: Vec<FinancialResourceYearly> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(resources.len(), all_res.len());

    // Assert
    for r in &resources {
        assert_ne!(r.year, next_year);
        assert!(all_res.contains(&r.base.id));
    }
}

#[sqlx::test]
async fn get_all_resources_returns_all_resources_of_all_years_with_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let prev_year = year - 1;
    let prev_year_id = app.insert_year(prev_year).await;
    let month1 = app.insert_random_month(year_id).await;
    let res1_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    let month2 = app.insert_random_month(year_id).await;
    let res1_month2 = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month2 = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Liability,
            DummyResourceType::LongTerm,
        )
        .await;

    let month3 = app.insert_random_month(prev_year_id).await;
    let res1_month3 = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month3 = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Liability,
            DummyResourceType::LongTerm,
        )
        .await;
    let all_res = [
        res1_month1.id,
        res2_month1.id,
        res1_month2.id,
        res2_month2.id,
        res1_month3.id,
        res2_month3.id,
    ];

    // Act
    let response = app.get_all_resources().await;
    assert!(response.status().is_success());

    let resources: Vec<FinancialResourceYearly> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(resources.len(), all_res.len());

    // Assert
    for r in &resources {
        assert!(all_res.contains(&r.base.id));
    }
}

#[sqlx::test]
async fn get_all_resources_returns_all_resources_of_all_years_ordered_by_year_and_by_month_in_res(
    pool: PgPool,
) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let prev_year = year - 1;
    let prev_year_id = app.insert_year(prev_year).await;
    let month = (2..12).fake();
    let prev_month = month - 1;
    let month1 = app.insert_month(year_id, month).await;
    let res1_month1 = app
        .insert_financial_resource(
            month1,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month1 = app
        .insert_financial_resource(
            month1,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;
    let month2 = app.insert_month(year_id, prev_month).await;
    app.insert_financial_resource_with_id_in_month(month2, res1_month1.id)
        .await;
    app.insert_financial_resource_with_id_in_month(month2, res2_month1.id)
        .await;
    let month3 = app.insert_random_month(prev_year_id).await;
    let res1_month3 = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month3 = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Liability,
            DummyResourceType::LongTerm,
        )
        .await;

    let all_res = [
        res1_month1.id,
        res2_month1.id,
        res1_month3.id,
        res2_month3.id,
    ];

    // Act
    let response = app.get_all_resources().await;
    assert!(response.status().is_success());

    let resources: Vec<FinancialResourceYearly> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(resources.len(), all_res.len());
    // Resource of previous year should be first
    let prev_year_res = [res1_month3.id, res2_month3.id];
    assert!(prev_year_res.contains(&resources[0].base.id));
    assert!(prev_year_res.contains(&resources[1].base.id));
}

#[sqlx::test]
async fn get_resources_returns_an_empty_list_even_if_year_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let year = Date().fake::<NaiveDate>().year();
    let response = app.get_resources(year).await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_resources_returns_an_empty_list_when_nothing_in_database(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_resources(year).await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_resources_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    // Sabotage the database
    sqlx::query!("ALTER TABLE balance_sheet_months DROP COLUMN month;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.get_resources(year).await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn get_resources_returns_balance_of_all_resources_in_all_months_of_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month1 = app.insert_random_month(year_id).await;
    let res1_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;
    let month2 = app.insert_month(year_id, month1.1.pred() as i16).await;
    app.insert_financial_resource_with_id_in_month(month2, res1_month1.id)
        .await;
    app.insert_financial_resource_with_id_in_month(month2, res2_month1.id)
        .await;

    let month3 = app.insert_random_month(year_id).await;
    let res1_month3 = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month3 = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Liability,
            DummyResourceType::LongTerm,
        )
        .await;

    let all_res = [
        res1_month1.id,
        res2_month1.id,
        res1_month3.id,
        res2_month3.id,
    ];

    // Act
    let response = app.get_all_resources().await;
    assert!(response.status().is_success());

    let resources: Vec<FinancialResourceYearly> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(resources.len(), all_res.len());

    for res in resources {
        assert!(all_res.contains(&res.base.id));
        if res.base.id == res1_month1.id || res.base.id == res2_month1.id {
            assert_eq!(res.balance_per_month.len(), 2);
        } else {
            assert_eq!(res.balance_per_month.len(), 1);
        }
    }
}

#[sqlx::test]
async fn get_resources_returns_months_ordered_in_same_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month1 = app.insert_random_month(year_id).await;
    let res1_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let res2_month1 = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    let prev_month = month1.1.pred();
    let month2 = app.insert_month(year_id, prev_month as i16).await;
    app.insert_financial_resource_with_id_in_month(month2, res1_month1.id)
        .await;
    app.insert_financial_resource_with_id_in_month(month2, res2_month1.id)
        .await;

    let all_res = [res1_month1.id, res2_month1.id];

    // Act
    let response = app.get_all_resources().await;
    assert!(response.status().is_success());

    let resources: Vec<FinancialResourceYearly> =
        serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(resources.len(), all_res.len());

    // Then in resource months should be ordered from first to last
    assert_eq!(
        *resources[0].balance_per_month.first_key_value().unwrap().0 as i16,
        prev_month as i16
    );
    assert_eq!(
        *resources[0].balance_per_month.last_key_value().unwrap().0 as i16,
        month1.1 as i16
    );
    assert_eq!(
        *resources[1].balance_per_month.first_key_value().unwrap().0 as i16,
        prev_month as i16
    );
    assert_eq!(
        *resources[1].balance_per_month.last_key_value().unwrap().0 as i16,
        month1.1 as i16
    );
}

#[sqlx::test]
async fn post_resources_returns_a_201_for_valid_body_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = app.insert_random_month(year_id).await;

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
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month.1, Faker.fake::<i64>());
    let body = Body {
        balance_per_month,
        year,
        ..Faker.fake()
    };

    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test]
async fn post_resources_returns_a_422_for_invalid_month_number(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    #[derive(Debug, Clone, Serialize, Dummy)]
    struct Body {
        name: String,
        category: DummyResourceCategory,
        #[serde(rename = "type")]
        r_type: DummyResourceType,
        editable: bool,
        year: i32,
        balance_per_month: BTreeMap<i16, i64>,
    }
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert((13..i16::MAX).fake(), Faker.fake::<i64>());
    let body = Body {
        balance_per_month,
        year,
        ..Faker.fake()
    };

    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn post_resources_returns_a_422_for_wrong_body_attributes(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct Body {
        name: String,
        category: DummyResourceCategory,
        #[serde(rename = "type")]
        r_type: DummyResourceType,
        editableeeeeeeeeeeeeeeeeeeee: bool,
        year: i32,
        balance_per_month: BTreeMap<i16, i64>,
    }
    let body = Body {
        year,
        ..Faker.fake()
    };

    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn post_resources_returns_a_422_for_missing_body_attributes(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct Body {
        name: String,
        category: DummyResourceCategory,
        #[serde(rename = "type")]
        r_type: DummyResourceType,
        // editable: bool,
        year: i32,
        balance_per_month: BTreeMap<i16, i64>,
    }
    let body = Body {
        year,
        ..Faker.fake()
    };

    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn post_resources_returns_a_422_for_wrong_body_attribute_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize, Dummy)]
    struct Body {
        name: i64,
        category: DummyResourceCategory,
        #[serde(rename = "type")]
        r_type: DummyResourceType,
        editable: bool,
        year: i32,
        balance_per_month: BTreeMap<i16, i64>,
    }
    let body = Body {
        year,
        ..Faker.fake()
    };

    // Act
    let response = app.create_resource(&body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn post_resources_returns_a_400_for_empty_body(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .api_client
        .post(&format!("{}/api/balance_sheet/resources", &app.address))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn post_resources_returns_a_415_for_missing_json_content_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app
        .api_client
        .post(&format!("{}/api/balance_sheet/resources", &app.address))
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
async fn post_resources_persists_the_new_resource(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = app.insert_random_month(year_id).await;

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
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month.1, Faker.fake::<i64>());
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    app.create_resource(&body).await;

    // Assert
    let saved = sqlx::query!("SELECT * FROM balance_sheet_resources",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved resource.");
    assert_eq!(saved.name, body.name);
}

// TODO: What should we do?
// #[sqlx::test]
// async fn post_resources_returns_a_409_if_resource_already_exists(pool: PgPool) {
//     // Arange
//     let app = spawn_app(pool).await;
//     let year = Date().fake::<NaiveDate>().year();
//     let year_id = app.insert_year(year).await;
//     let month = (1..12).fake();
//     app.insert_month(year_id, month).await;

//     #[derive(Debug, Clone, Serialize)]
//     struct Body {
//         month: i16,
//     }
//     let body = Body { month };

//     // Act
//     let response = app.create_month(year, &body).await;

//     // Assert
//     assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
// }

#[sqlx::test]
async fn post_resources_persits_net_totals_for_month_and_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = app.insert_random_month(year_id).await;
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
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(month.1, Faker.fake::<i64>());
    let body = Body {
        year,
        balance_per_month,
        ..Faker.fake()
    };

    // Act
    app.create_resource(&body).await;

    // Assert
    let saved_month_net_totals = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE month_id = $1",
        month.0
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
async fn post_resources_updates_net_totals_if_previous_month_exists(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = (2..12).fake();
    let month_id = app.insert_month(year_id, month).await;

    let prev_month = month - 1;
    let month2_id = app.insert_month(year_id, prev_month).await;
    let month_net_total_assets = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Portfolio)
        .await;

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
    let mut balance_per_month = BTreeMap::new();
    let month_balance = Faker.fake::<i32>() as i64;
    balance_per_month.insert(month.try_into().unwrap(), month_balance);
    let body = Body {
        balance_per_month,
        category: DummyResourceCategory::Asset,
        r_type: DummyResourceType::Cash,
        year,
        ..Faker.fake()
    };

    // Act
    app.create_resource(&body).await;

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
