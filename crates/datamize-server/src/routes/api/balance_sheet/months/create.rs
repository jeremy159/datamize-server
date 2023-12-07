use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::{FinancialResourceMonthly, Month, MonthNum, NetTotal};
use fake::{faker::chrono::en::Date, Fake, Faker};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::months::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct CreateBody {
    pub month: MonthNum,
}

fn are_equal(a: &Month, b: &Month) {
    assert_eq!(a.month, b.month);
    assert_eq!(a.year, b.year);
    assert_eq!(a.net_assets.total, b.net_assets.total);
    assert_eq!(a.net_assets.balance_var, b.net_assets.balance_var);
    assert_eq!(a.net_assets.percent_var, b.net_assets.percent_var);
    assert_eq!(a.net_portfolio.total, b.net_assets.total);
    assert_eq!(a.net_portfolio.balance_var, b.net_portfolio.balance_var);
    assert_eq!(a.net_portfolio.percent_var, b.net_portfolio.percent_var);
    assert_eq!(a.resources, b.resources);
}

async fn check_create(
    pool: SqlitePool,
    year: Option<i32>,
    create_year: bool,
    body: Option<CreateBody>,
    expected_status: StatusCode,
    expected_resp: Option<Month>,
) {
    let context = TestContext::setup(pool);

    if let Some(year) = year {
        if create_year {
            context.insert_year(year).await;
        }
    }
    let year = year.unwrap_or(Date().fake::<NaiveDate>().year());

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/years/{:?}/months", year))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(mut expected) = expected_resp {
        expected.compute_net_totals();
        let body: Month = serde_json::from_slice(&body).unwrap();
        are_equal(&body, &expected);

        let saved = context
            .get_month(expected.month, expected.year)
            .await
            .unwrap();
        assert_eq!(body, saved);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_month(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    let year = Date().fake::<NaiveDate>().year();
    let month: Month = Month {
        month: body.month,
        year,
        net_assets: NetTotal {
            total: 0,
            balance_var: 0,
            percent_var: 0.0,
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: 0,
            percent_var: 0.0,
            ..Faker.fake()
        },
        resources: vec![],
        ..Faker.fake()
    };

    check_create(
        pool,
        Some(year),
        true,
        Some(body.clone()),
        StatusCode::CREATED,
        Some(month),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_year_does_not_exist(pool: SqlitePool) {
    check_create(
        pool,
        None,
        false,
        Some(Faker.fake()),
        StatusCode::NOT_FOUND,
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_409_when_month_already_exists(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    let year = Date().fake::<NaiveDate>().year();
    {
        let context = TestContext::setup(pool.clone());
        context.insert_year(year).await;
        let month = Month {
            month: body.month,
            year,
            ..Faker.fake()
        };
        let month = Month {
            resources: month
                .resources
                .into_iter()
                .map(|r| FinancialResourceMonthly {
                    year,
                    month: month.month,
                    ..r
                })
                .collect(),
            ..month
        };

        context.set_month(&month, year).await;
    }
    check_create(
        pool,
        Some(year),
        false,
        Some(body),
        StatusCode::CONFLICT,
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_if_prev_month_exists(pool: SqlitePool) {
    let month: i16 = (2..12).fake();
    let year = Date().fake::<NaiveDate>().year();
    let body = CreateBody {
        month: month.try_into().unwrap(),
    };
    let prev_month = Month {
        month: (month - 1).try_into().unwrap(),
        year,
        ..Faker.fake()
    };
    let mut prev_month = Month {
        resources: prev_month
            .resources
            .into_iter()
            .map(|r| FinancialResourceMonthly {
                year: prev_month.year,
                month: prev_month.month,
                ..r
            })
            .collect(),
        ..prev_month
    };
    prev_month.compute_net_totals();

    let month = Month {
        month: body.month,
        year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_month.net_assets.total,
            percent_var: if prev_month.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_month.net_portfolio.total,
            percent_var: if prev_month.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        resources: vec![],
        ..Faker.fake()
    };

    {
        let context = TestContext::setup(pool.clone());
        context.insert_year(year).await;
        context.set_month(&prev_month, year).await;
    }
    check_create(
        pool,
        Some(year),
        false,
        Some(body),
        StatusCode::CREATED,
        Some(month),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_if_prev_month_exists_in_prev_year(pool: SqlitePool) {
    let year = Date().fake::<NaiveDate>().year();
    let body = CreateBody {
        month: MonthNum::January,
    };
    let prev_month = Month {
        month: MonthNum::December,
        year: year - 1,
        ..Faker.fake()
    };
    let mut prev_month = Month {
        resources: prev_month
            .resources
            .into_iter()
            .map(|r| FinancialResourceMonthly {
                year: prev_month.year,
                month: prev_month.month,
                ..r
            })
            .collect(),
        ..prev_month
    };
    prev_month.compute_net_totals();

    let month = Month {
        month: body.month,
        year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_month.net_assets.total,
            percent_var: if prev_month.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_month.net_portfolio.total,
            percent_var: if prev_month.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        resources: vec![],
        ..Faker.fake()
    };

    {
        let context = TestContext::setup(pool.clone());
        context.insert_year(year - 1).await;
        context.set_month(&prev_month, year - 1).await;
    }
    check_create(
        pool,
        Some(year),
        true,
        Some(body),
        StatusCode::CREATED,
        Some(month),
    )
    .await;
}
