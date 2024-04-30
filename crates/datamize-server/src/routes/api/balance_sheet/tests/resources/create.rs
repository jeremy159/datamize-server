use std::collections::BTreeMap;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::{
    testutils::{financial_resource_yearly_equal_without_id, NUM_MONTHS},
    BaseFinancialResource, FinancialResourceType, FinancialResourceYearly, MonthNum, Uuid,
    YearlyBalances,
};
use fake::{Dummy, Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::{assert_eq, assert_ne};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::resources::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CreateBody {
    pub name: String,
    #[serde(with = "datamize_domain::string")]
    pub resource_type: FinancialResourceType,
    pub balances: BTreeMap<i32, BTreeMap<MonthNum, Option<i64>>>,
    pub ynab_account_ids: Option<Vec<Uuid>>,
    pub external_account_ids: Option<Vec<Uuid>>,
}

impl fake::Dummy<fake::Faker> for CreateBody {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        let name = Fake::fake_with_rng(&Faker, rng);
        let resource_type = Fake::fake_with_rng(&Faker, rng);

        let mut balances = BTreeMap::new();
        let len = (1..10).fake_with_rng(rng);
        for _ in 0..len {
            let len_values = (1..NUM_MONTHS).fake_with_rng(rng);
            let mut month_balances = BTreeMap::new();
            for _ in 0..len_values {
                let month = Fake::fake_with_rng(&Faker, rng);
                month_balances.insert(month, Some(Fake::fake_with_rng(&(-1000000..1000000), rng)));
            }
            balances.insert(Fake::fake_with_rng(&(1000..3000), rng), month_balances);
        }
        let ynab_account_ids = Fake::fake_with_rng(&Faker, rng);
        let external_account_ids = Fake::fake_with_rng(&Faker, rng);

        Self {
            name,
            resource_type,
            balances,
            ynab_account_ids,
            external_account_ids,
        }
    }
}

async fn check_create(
    pool: SqlitePool,
    body: CreateBody,
    expected_status: StatusCode,
    expected_resp: Option<FinancialResourceYearly>,
) {
    let context = TestContext::setup(pool).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resources")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = expected_resp {
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        financial_resource_yearly_equal_without_id(&body, &expected);

        if !expected.is_empty() {
            // Persits the resource
            let saved = context.get_res_by_name(&expected.base.name).await.unwrap();
            assert!(!saved.is_empty());
            assert_eq!(body, saved);

            // Creates all months that were not created
            for (year, month, _) in expected.iter_balances() {
                let saved_month = context.get_month(month, year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                // Since net_assets are computed from all resources' type
                assert_ne!(saved_month.net_assets().total, 0);
            }
        }

        // Creating the resource also computed net assets of the year
        let saved_years = context.get_years().await;
        assert!(saved_years.is_ok());
        let saved_years = saved_years.unwrap();
        for saved_year in saved_years {
            assert_ne!(saved_year.net_assets().total, 0);
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_resource(pool: SqlitePool) {
    let mut body = CreateBody {
        balances: BTreeMap::new(),
        ..Faker.fake()
    };
    let current_date = Faker.fake::<NaiveDate>();
    let month: MonthNum = current_date.month().try_into().unwrap();
    let year = current_date.year();
    body.balances
        .entry(year)
        .or_default()
        .insert(month, Some((-1000000..1000000).fake()));

    let body_cloned = body.clone();

    let res = FinancialResourceYearly {
        balances: body_cloned.balances,
        base: BaseFinancialResource {
            name: body_cloned.name,
            resource_type: body_cloned.resource_type,
            ynab_account_ids: body_cloned.ynab_account_ids,
            external_account_ids: body_cloned.external_account_ids,
            ..Faker.fake()
        },
    };

    check_create(pool, body, StatusCode::CREATED, Some(res)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_409_when_resource_already_exists(pool: SqlitePool) {
    let mut resource = FinancialResourceYearly::new(
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
    );
    let current_date = Faker.fake::<NaiveDate>();
    let month = current_date.month().try_into().unwrap();
    let year = current_date.year();
    resource.insert_balance(year, month, (-1000000..1000000).fake());

    let res = resource.clone();
    let body = CreateBody {
        name: res.base.name,
        resource_type: res.base.resource_type,
        balances: res.balances,
        ynab_account_ids: res.base.ynab_account_ids,
        external_account_ids: res.base.external_account_ids,
    };
    let context = TestContext::setup(pool.clone()).await;
    context.insert_year(year).await;
    context.insert_month(month, year).await;
    context.set_resources(&[resource]).await;

    check_create(pool, body, StatusCode::CONFLICT, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_422_for_invalid_body_format_data(pool: SqlitePool) {
    let context = TestContext::setup(pool).await;

    #[derive(Debug, Clone, Serialize, Dummy)]
    struct ReqBody {
        pub name: String,
        #[serde(with = "datamize_domain::string")]
        pub resource_type: FinancialResourceType,
    }
    let body = Faker.fake::<ReqBody>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resources")
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
    let context = TestContext::setup(pool).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resources")
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
    let context = TestContext::setup(pool).await;

    let body = Faker.fake::<CreateBody>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/resources")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
