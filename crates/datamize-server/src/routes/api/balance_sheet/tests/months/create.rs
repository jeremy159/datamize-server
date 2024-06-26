use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::{Month, MonthNum, NetTotal, NetTotals, Uuid};
use fake::{faker::chrono::en::Date, Dummy, Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::months::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone, fake::Dummy)]
struct CreateBody {
    pub month: MonthNum,
}

fn are_equal(a: &Month, b: &Month) {
    assert_eq!(a.month, b.month);
    assert_eq!(a.year, b.year);
    assert_eq!(a.net_assets().total, b.net_assets().total);
    assert_eq!(a.net_assets().balance_var, b.net_assets().balance_var);
    assert_eq!(a.net_assets().percent_var, b.net_assets().percent_var);
    assert_eq!(a.net_portfolio().total, b.net_assets().total);
    assert_eq!(a.net_portfolio().balance_var, b.net_portfolio().balance_var);
    assert_eq!(a.net_portfolio().percent_var, b.net_portfolio().percent_var);
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

    let body = response.into_body().collect().await.unwrap().to_bytes();

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
        net_totals: NetTotals::default(),
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
    let mut prev_month = Month {
        month: (month - 1).try_into().unwrap(),
        year,
        ..Faker.fake()
    };
    prev_month.compute_net_totals();

    let month = Month {
        month: body.month,
        year,
        net_totals: NetTotals {
            assets: NetTotal {
                total: 0,
                balance_var: -prev_month.net_assets().total,
                percent_var: if prev_month.net_assets().total == 0 {
                    0.0
                } else {
                    -1.0
                },
                ..Faker.fake()
            },
            portfolio: NetTotal {
                total: 0,
                balance_var: -prev_month.net_portfolio().total,
                percent_var: if prev_month.net_portfolio().total == 0 {
                    0.0
                } else {
                    -1.0
                },
                ..Faker.fake()
            },
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
    let mut prev_month = Month {
        month: MonthNum::December,
        year: year - 1,
        ..Faker.fake()
    };
    prev_month.compute_net_totals();

    let month = Month {
        month: body.month,
        year,
        net_totals: NetTotals {
            assets: NetTotal {
                total: 0,
                balance_var: -prev_month.net_assets().total,
                percent_var: if prev_month.net_assets().total == 0 {
                    0.0
                } else {
                    -1.0
                },
                ..Faker.fake()
            },
            portfolio: NetTotal {
                total: 0,
                balance_var: -prev_month.net_portfolio().total,
                percent_var: if prev_month.net_portfolio().total == 0 {
                    0.0
                } else {
                    -1.0
                },
                ..Faker.fake()
            },
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

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_year_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/years/{}/months", Faker.fake::<Uuid>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_422_for_invalid_body_format_data(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    #[derive(Debug, Clone, Serialize, Dummy)]
    struct ReqBody {
        pub month: String,
    }
    let body = Faker.fake::<ReqBody>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/years/{}/months", Faker.fake::<i32>()))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_empty_body(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/years/{}/months", Faker.fake::<i32>()))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_415_for_missing_json_content_type(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let body = Faker.fake::<CreateBody>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/years/{}/months", Faker.fake::<i32>()))
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
