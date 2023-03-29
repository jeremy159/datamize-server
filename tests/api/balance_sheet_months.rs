use chrono::{Datelike, NaiveDate};
use datamize::domain::{Month, ResourceCategory};
use fake::faker::chrono::en::Date;
use fake::Fake;
use serde::Serialize;
use sqlx::PgPool;

use crate::dummy_types::{DummyNetTotalType, DummyResourceCategory, DummyResourceType};
use crate::helpers::spawn_app;

#[sqlx::test]
async fn get_months_returns_an_empty_list_even_if_year_does_not_exist(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let year = Date().fake::<NaiveDate>().year();
    let response = app.get_months(year).await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_months_returns_an_empty_list_when_nothing_in_database(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_months(year).await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_months_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
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
    let response = app.get_months(year).await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn get_months_returns_net_totals_and_resources_of_all_months_of_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month1 = app.insert_random_month(year_id).await;
    let month1_net_total_assets = app
        .insert_month_net_total(month1.0, DummyNetTotalType::Asset)
        .await;
    let month1_net_total_portfolio = app
        .insert_month_net_total(month1.0, DummyNetTotalType::Portfolio)
        .await;
    let month1_first_res = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month1_second_res = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    let month2 = app.insert_random_month(year_id).await;
    let month2_net_total_assets = app
        .insert_month_net_total(month2.0, DummyNetTotalType::Asset)
        .await;
    let month2_net_total_portfolio = app
        .insert_month_net_total(month2.0, DummyNetTotalType::Portfolio)
        .await;
    let month2_first_res = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month2_second_res = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    let response = app.get_months(year).await;
    assert!(response.status().is_success());

    let months: Vec<Month> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    for m in &months {
        if m.id == month1.0 {
            assert_eq!(m.net_assets.id, month1_net_total_assets.id);
            assert_eq!(m.net_assets.total, month1_net_total_assets.total as i64);
            assert_eq!(m.net_portfolio.id, month1_net_total_portfolio.id);
            assert_eq!(
                m.net_portfolio.total,
                month1_net_total_portfolio.total as i64
            );

            for r in &m.resources {
                if r.base.category == ResourceCategory::Asset {
                    assert_eq!(r.base.id, month1_first_res.id);
                    assert_eq!(r.balance, month1_first_res.balance);
                } else if r.base.category == ResourceCategory::Liability {
                    assert_eq!(r.base.id, month1_second_res.id);
                    assert_eq!(r.balance, month1_second_res.balance);
                }
            }
        } else if m.id == month2.0 {
            assert_eq!(m.net_assets.id, month2_net_total_assets.id);
            assert_eq!(m.net_assets.total, month2_net_total_assets.total as i64);
            assert_eq!(m.net_portfolio.id, month2_net_total_portfolio.id);
            assert_eq!(
                m.net_portfolio.total,
                month2_net_total_portfolio.total as i64
            );

            for r in &m.resources {
                if r.base.category == ResourceCategory::Asset {
                    assert_eq!(r.base.id, month2_first_res.id);
                    assert_eq!(r.balance, month2_first_res.balance);
                } else if r.base.category == ResourceCategory::Liability {
                    assert_eq!(r.base.id, month2_second_res.id);
                    assert_eq!(r.balance, month2_second_res.balance);
                }
            }
        }
    }
}

#[sqlx::test]
async fn post_months_returns_a_201_for_valid_body_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        month: i16,
    }
    let body = Body {
        month: (1..12).fake(),
    };

    // Act
    let response = app.create_month(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[sqlx::test]
async fn post_months_returns_a_422_for_invalid_month_number(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        month: i16,
    }
    let body = Body {
        month: (13..i16::MAX).fake(),
    };

    // Act
    let response = app.create_month(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn post_months_returns_a_400_for_empty_body(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();

    // Act
    let response = app
        .api_client
        .post(&format!(
            "{}/api/balance_sheet/years/{}/months",
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
async fn post_months_returns_a_415_for_missing_json_content_type(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();

    // Act
    let response = app
        .api_client
        .post(&format!(
            "{}/api/balance_sheet/years/{}/months",
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
async fn post_months_persists_the_new_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        month: i16,
    }
    let body = Body {
        month: (1..12).fake(),
    };

    // Act
    app.create_month(year, &body).await;

    // Assert
    let saved = sqlx::query!("SELECT month FROM balance_sheet_months",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved month.");
    assert_eq!(saved.month, body.month);
}

#[sqlx::test]
async fn post_months_returns_a_409_if_month_already_exists(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = (1..12).fake();
    app.insert_month(year_id, month).await;

    #[derive(Debug, Clone, Serialize)]
    struct Body {
        month: i16,
    }
    let body = Body { month };

    // Act
    let response = app.create_month(year, &body).await;

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[sqlx::test]
async fn post_months_persits_net_totals_for_new_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    #[derive(Debug, Clone, Serialize)]
    struct Body {
        month: i16,
    }
    let body = Body {
        month: (1..12).fake(),
    };

    // Act
    let response = app.create_month(year, &body).await;
    let month: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    let saved_net_totals = sqlx::query!(
        "SELECT * FROM balance_sheet_net_totals_months WHERE month_id = $1",
        month.id
    )
    .fetch_all(&app.db_pool)
    .await
    .expect("Failed to fetch net totals.");
    assert!(!saved_net_totals.is_empty());
}

#[sqlx::test]
async fn post_months_updates_net_totals_if_previous_month_exists(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let month = (2..12).fake();

    let prev_month = month - 1;
    let month2_id = app.insert_month(year_id, prev_month).await;
    let month_net_total_assets = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Portfolio)
        .await;

    #[derive(Debug, Clone, Serialize)]
    struct Body {
        month: i16,
    }
    let body = Body { month };

    // Act
    let response = app.create_month(year, &body).await;
    let month: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(
        month.net_assets.balance_var,
        -month_net_total_assets.total as i64
    );
    assert_eq!(month.net_assets.percent_var, -1.0);
    assert_eq!(
        month.net_portfolio.balance_var,
        -month_net_total_portfolio.total as i64
    );
    assert_eq!(month.net_portfolio.percent_var, -1.0);
}

#[sqlx::test]
async fn post_months_updates_net_totals_if_previous_month_exists_in_prev_year(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;
    let month = 1; // January

    let prev_year = year - 1;
    let prev_year_id = app.insert_year(prev_year).await;
    let prev_month = 12; // December of prev year
    let month2_id = app.insert_month(prev_year_id, prev_month).await;
    let month_net_total_assets = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Asset)
        .await;
    let month_net_total_portfolio = app
        .insert_month_net_total(month2_id, DummyNetTotalType::Portfolio)
        .await;

    #[derive(Debug, Clone, Serialize)]
    struct Body {
        month: i16,
    }
    let body = Body { month };

    // Act
    let response = app.create_month(year, &body).await;
    let month: Month = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(
        month.net_assets.balance_var,
        -month_net_total_assets.total as i64
    );
    assert_eq!(month.net_assets.percent_var, -1.0);
    assert_eq!(
        month.net_portfolio.balance_var,
        -month_net_total_portfolio.total as i64
    );
    assert_eq!(month.net_portfolio.percent_var, -1.0);
}

#[sqlx::test]
async fn get_all_months_returns_an_empty_list_when_nothing_in_database(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;

    // Act
    let response = app.get_all_months().await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_all_months_fails_if_there_is_a_fatal_database_error(pool: PgPool) {
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
    let response = app.get_all_months().await;

    // Assert
    assert_eq!(
        response.status(),
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[sqlx::test]
async fn get_all_months_returns_an_empty_list_even_if_year_is_in_db(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    app.insert_year(year).await;

    // Act
    let response = app.get_all_months().await;

    // Assert
    assert!(response.status().is_success());
    let value: serde_json::Value = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert!(value.is_array());
}

#[sqlx::test]
async fn get_all_months_returns_all_months_of_only_years_with_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let next_year = year + 1;
    app.insert_year(next_year).await;
    let month1 = app.insert_random_month(year_id).await;
    let month1_net_total_assets = app
        .insert_month_net_total(month1.0, DummyNetTotalType::Asset)
        .await;
    let month1_net_total_portfolio = app
        .insert_month_net_total(month1.0, DummyNetTotalType::Portfolio)
        .await;
    let month1_first_res = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month1_second_res = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    let month2 = app.insert_random_month(year_id).await;
    let month2_net_total_assets = app
        .insert_month_net_total(month2.0, DummyNetTotalType::Asset)
        .await;
    let month2_net_total_portfolio = app
        .insert_month_net_total(month2.0, DummyNetTotalType::Portfolio)
        .await;
    let month2_first_res = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month2_second_res = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    let response = app.get_all_months().await;
    assert!(response.status().is_success());

    let months: Vec<Month> = serde_json::from_str(&response.text().await.unwrap()).unwrap();
    assert_eq!(months.len(), 2);

    // Assert
    for m in &months {
        assert_ne!(m.year, next_year);
        if m.id == month1.0 {
            assert_eq!(m.net_assets.id, month1_net_total_assets.id);
            assert_eq!(m.net_assets.total, month1_net_total_assets.total as i64);
            assert_eq!(m.net_portfolio.id, month1_net_total_portfolio.id);
            assert_eq!(
                m.net_portfolio.total,
                month1_net_total_portfolio.total as i64
            );

            for r in &m.resources {
                if r.base.category == ResourceCategory::Asset {
                    assert_eq!(r.base.id, month1_first_res.id);
                    assert_eq!(r.balance, month1_first_res.balance);
                } else if r.base.category == ResourceCategory::Liability {
                    assert_eq!(r.base.id, month1_second_res.id);
                    assert_eq!(r.balance, month1_second_res.balance);
                }
            }
        } else if m.id == month2.0 {
            assert_eq!(m.net_assets.id, month2_net_total_assets.id);
            assert_eq!(m.net_assets.total, month2_net_total_assets.total as i64);
            assert_eq!(m.net_portfolio.id, month2_net_total_portfolio.id);
            assert_eq!(
                m.net_portfolio.total,
                month2_net_total_portfolio.total as i64
            );

            for r in &m.resources {
                if r.base.category == ResourceCategory::Asset {
                    assert_eq!(r.base.id, month2_first_res.id);
                    assert_eq!(r.balance, month2_first_res.balance);
                } else if r.base.category == ResourceCategory::Liability {
                    assert_eq!(r.base.id, month2_second_res.id);
                    assert_eq!(r.balance, month2_second_res.balance);
                }
            }
        }
    }
}

#[sqlx::test]
async fn get_all_months_returns_all_months_of_all_years_with_data(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let prev_year = year - 1;
    let prev_year_id = app.insert_year(prev_year).await;
    let month1 = app.insert_random_month(year_id).await;
    let month1_net_total_assets = app
        .insert_month_net_total(month1.0, DummyNetTotalType::Asset)
        .await;
    let month1_net_total_portfolio = app
        .insert_month_net_total(month1.0, DummyNetTotalType::Portfolio)
        .await;
    let month1_first_res = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month1_second_res = app
        .insert_financial_resource(
            month1.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    let month2 = app.insert_random_month(year_id).await;
    let month2_net_total_assets = app
        .insert_month_net_total(month2.0, DummyNetTotalType::Asset)
        .await;
    let month2_net_total_portfolio = app
        .insert_month_net_total(month2.0, DummyNetTotalType::Portfolio)
        .await;
    let month2_first_res = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month2_second_res = app
        .insert_financial_resource(
            month2.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    let month3 = app.insert_random_month(prev_year_id).await;
    let month3_net_total_assets = app
        .insert_month_net_total(month3.0, DummyNetTotalType::Asset)
        .await;
    let month3_net_total_portfolio = app
        .insert_month_net_total(month3.0, DummyNetTotalType::Portfolio)
        .await;
    let month3_first_res = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Asset,
            DummyResourceType::Cash,
        )
        .await;
    let month3_second_res = app
        .insert_financial_resource(
            month3.0,
            DummyResourceCategory::Liability,
            DummyResourceType::Cash,
        )
        .await;

    // Act
    let response = app.get_all_months().await;
    assert!(response.status().is_success());

    let months: Vec<Month> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(months.len(), 3);

    for m in &months {
        if m.id == month1.0 {
            assert_eq!(m.net_assets.id, month1_net_total_assets.id);
            assert_eq!(m.net_assets.total, month1_net_total_assets.total as i64);
            assert_eq!(m.net_portfolio.id, month1_net_total_portfolio.id);
            assert_eq!(
                m.net_portfolio.total,
                month1_net_total_portfolio.total as i64
            );

            for r in &m.resources {
                if r.base.category == ResourceCategory::Asset {
                    assert_eq!(r.base.id, month1_first_res.id);
                    assert_eq!(r.balance, month1_first_res.balance);
                } else if r.base.category == ResourceCategory::Liability {
                    assert_eq!(r.base.id, month1_second_res.id);
                    assert_eq!(r.balance, month1_second_res.balance);
                }
            }
        } else if m.id == month2.0 {
            assert_eq!(m.net_assets.id, month2_net_total_assets.id);
            assert_eq!(m.net_assets.total, month2_net_total_assets.total as i64);
            assert_eq!(m.net_portfolio.id, month2_net_total_portfolio.id);
            assert_eq!(
                m.net_portfolio.total,
                month2_net_total_portfolio.total as i64
            );

            for r in &m.resources {
                if r.base.category == ResourceCategory::Asset {
                    assert_eq!(r.base.id, month2_first_res.id);
                    assert_eq!(r.balance, month2_first_res.balance);
                } else if r.base.category == ResourceCategory::Liability {
                    assert_eq!(r.base.id, month2_second_res.id);
                    assert_eq!(r.balance, month2_second_res.balance);
                }
            }
        } else if m.id == month3.0 {
            assert_eq!(m.net_assets.id, month3_net_total_assets.id);
            assert_eq!(m.net_assets.total, month3_net_total_assets.total as i64);
            assert_eq!(m.net_portfolio.id, month3_net_total_portfolio.id);
            assert_eq!(
                m.net_portfolio.total,
                month3_net_total_portfolio.total as i64
            );

            for r in &m.resources {
                if r.base.category == ResourceCategory::Asset {
                    assert_eq!(r.base.id, month3_first_res.id);
                    assert_eq!(r.balance, month3_first_res.balance);
                } else if r.base.category == ResourceCategory::Liability {
                    assert_eq!(r.base.id, month3_second_res.id);
                    assert_eq!(r.balance, month3_second_res.balance);
                }
            }
        }
    }
}

#[sqlx::test]
async fn get_all_months_returns_all_months_of_all_years_ordered_by_year_then_month(pool: PgPool) {
    // Arange
    let app = spawn_app(pool).await;
    let year = Date().fake::<NaiveDate>().year();
    let year_id = app.insert_year(year).await;
    let prev_year = year - 1;
    let prev_year_id = app.insert_year(prev_year).await;
    let month = (2..12).fake();
    let prev_month = month - 1;
    let month1 = app.insert_month(year_id, month).await;
    app.insert_month_net_total(month1, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month1, DummyNetTotalType::Portfolio)
        .await;
    let month2 = app.insert_month(year_id, prev_month).await;
    app.insert_month_net_total(month2, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month2, DummyNetTotalType::Portfolio)
        .await;
    let month3 = app.insert_random_month(prev_year_id).await;
    app.insert_month_net_total(month3.0, DummyNetTotalType::Asset)
        .await;
    app.insert_month_net_total(month3.0, DummyNetTotalType::Portfolio)
        .await;

    // Act
    let response = app.get_all_months().await;
    assert!(response.status().is_success());

    let months: Vec<Month> = serde_json::from_str(&response.text().await.unwrap()).unwrap();

    // Assert
    assert_eq!(months.len(), 3);
    // Months of previous year should be first
    assert_eq!(months[0].id, month3.0);
    // Then previous month of same year
    assert_eq!(months[1].id, month2);
    assert_eq!(months[2].id, month1);
}
