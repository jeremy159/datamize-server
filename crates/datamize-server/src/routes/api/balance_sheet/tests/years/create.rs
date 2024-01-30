use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{NetTotal, NetTotalType, Year};
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::{assert_eq, assert_ne};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::years::testutils::{
    correctly_stub_year, TestContext,
};

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct CreateBody {
    pub year: i32,
}

fn are_equal(a: &Year, b: &Year) {
    assert_eq!(a.year, b.year);
    assert_eq!(a.net_assets.total, b.net_assets.total);
    assert_eq!(a.net_assets.balance_var, b.net_assets.balance_var);
    assert_eq!(a.net_assets.percent_var, b.net_assets.percent_var);
    assert_eq!(a.net_portfolio.total, b.net_assets.total);
    assert_eq!(a.net_portfolio.balance_var, b.net_portfolio.balance_var);
    assert_eq!(a.net_portfolio.percent_var, b.net_portfolio.percent_var);
    assert_eq!(a.months, b.months);
}

async fn check_create(
    pool: SqlitePool,
    body: Option<CreateBody>,
    expected_status: StatusCode,
    expected_resp: Option<Year>,
) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/years")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = expected_resp {
        // expected.compute_net_totals();
        let body: Year = serde_json::from_slice(&body).unwrap();
        are_equal(&body, &expected);

        let saved = context.get_year(expected.year).await.unwrap();
        assert_eq!(body, saved);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_year(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    let year = Year {
        year: body.year,
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
        months: vec![],
        ..Faker.fake()
    };

    check_create(pool, Some(body.clone()), StatusCode::CREATED, Some(year)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_409_when_year_already_exists(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    {
        let context = TestContext::setup(pool.clone());
        let year = Year {
            year: body.year,
            ..Faker.fake()
        };
        let year = correctly_stub_year(Some(year)).unwrap();
        context.set_year(&year).await;
    }
    check_create(pool, Some(body), StatusCode::CONFLICT, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_if_prev_year_exists(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    let prev_year = Year {
        year: body.year - 1,
        ..Faker.fake()
    };
    let prev_year = correctly_stub_year(Some(prev_year)).unwrap();

    let year = Year {
        year: body.year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_year.net_assets.total,
            percent_var: if prev_year.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_year.net_portfolio.total,
            percent_var: if prev_year.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        months: vec![],
        ..Faker.fake()
    };

    {
        let context = TestContext::setup(pool.clone());
        context.set_year(&prev_year).await;
    }
    check_create(pool, Some(body), StatusCode::CREATED, Some(year)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_of_prev_and_next_years_if_exists(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    let prev_year = Year {
        year: body.year - 1,
        ..Faker.fake()
    };
    let prev_year = correctly_stub_year(Some(prev_year)).unwrap();

    let next_year = Year {
        year: body.year + 1,
        ..Faker.fake()
    };
    let next_year = correctly_stub_year(Some(next_year)).unwrap();

    let year = Year {
        year: body.year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_year.net_assets.total,
            percent_var: if prev_year.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_year.net_portfolio.total,
            percent_var: if prev_year.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        months: vec![],
        ..Faker.fake()
    };

    let context = TestContext::setup(pool.clone());
    context.set_year(&prev_year).await;
    let next_year_id = context.set_year(&next_year).await;

    check_create(pool, Some(body), StatusCode::CREATED, Some(year)).await;

    let saved_next_net_totals = context.get_net_totals(next_year_id).await.unwrap();
    for next_nt in saved_next_net_totals {
        match next_nt.net_type {
            NetTotalType::Asset => {
                assert_ne!(next_nt.balance_var, next_year.net_assets.balance_var);
                assert_ne!(next_nt.percent_var, next_year.net_assets.percent_var);
            }
            NetTotalType::Portfolio => {
                assert_ne!(next_nt.balance_var, next_year.net_portfolio.balance_var);
                assert_ne!(next_nt.percent_var, next_year.net_portfolio.percent_var);
            }
        };
    }
}
