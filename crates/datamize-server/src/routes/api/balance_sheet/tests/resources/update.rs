use std::collections::{BTreeMap, HashSet};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::{
    testutils::NUM_MONTHS, BaseFinancialResource, FinancialResourceType, FinancialResourceYearly,
    MonthNum, Uuid, YearlyBalances,
};
use fake::{Dummy, Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::{assert_eq, assert_ne};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::resources::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct UpdateBody {
    pub id: Uuid,
    pub name: String,
    #[serde(with = "datamize_domain::string")]
    pub resource_type: FinancialResourceType,
    pub balances: BTreeMap<i32, BTreeMap<MonthNum, Option<i64>>>,
    pub ynab_account_ids: Option<Vec<Uuid>>,
    pub external_account_ids: Option<Vec<Uuid>>,
}

impl fake::Dummy<fake::Faker> for UpdateBody {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        let id = Fake::fake_with_rng(&Faker, rng);
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
            id,
            name,
            resource_type,
            balances,
            ynab_account_ids,
            external_account_ids,
        }
    }
}

async fn check_update(
    pool: SqlitePool,
    req_body: UpdateBody,
    db_data: Option<FinancialResourceYearly>,
    expected_status: StatusCode,
    expected_resp: Option<FinancialResourceYearly>,
) {
    let context = TestContext::setup(pool).await;
    let mut checked_years = HashSet::<i32>::new();

    if let Some(ref db_data) = db_data {
        // Create all months and years
        for (year, month) in db_data.iter_months() {
            if !checked_years.contains(&year) {
                checked_years.insert(year);
                context.insert_year(year).await;
            }
            context.insert_month(month, year).await;
        }
        context.set_resource(db_data).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/resources/{:?}", req_body.id))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected_resp) = expected_resp {
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected_resp);

        if let Some(db_data) = db_data {
            // Make sure the requested body is not equal to the resource that was in the db. I.e. new balance per month should have updated something
            assert_ne!(req_body.name, db_data.base.name,);
        }

        // Make sure the update is persisted in db
        let saved = context.get_res(expected_resp.base.id).await.unwrap();
        assert_eq!(req_body.name, saved.base.name);
        assert_eq!(expected_resp.balances, saved.balances);

        if !expected_resp.is_empty() {
            // Creates all months that were not created
            for (year, month) in expected_resp.iter_months() {
                let saved_month = context.get_month(month, year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                if !saved_month.resources.is_empty() {
                    // Since net_assets are computed from all resources' type
                    assert_ne!(saved_month.net_assets().total, 0);
                }
            }
        }

        // Updating the resource also computed net assets of the year
        let saved_years = context.get_years().await;
        assert!(saved_years.is_ok());
        let saved_years = saved_years.unwrap();
        for saved_year in saved_years {
            if let Some(last_month) = saved_year.get_last_month() {
                assert_eq!(saved_year.net_assets().total, last_month.net_assets().total);
            }
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, Faker.fake(), None, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let mut db_data: FinancialResourceYearly = Faker.fake();
    db_data.clear_all_balances();
    let current_date = Faker.fake::<NaiveDate>();
    let month = current_date.month().try_into().unwrap();
    let year = current_date.year();
    db_data.insert_balance(year, month, (-1000000..1000000).fake());

    let db_data_cloned = db_data.clone();
    let mut body = UpdateBody {
        id: db_data_cloned.base.id,
        name: db_data_cloned.base.name,
        resource_type: db_data_cloned.base.resource_type,
        ynab_account_ids: db_data_cloned.base.ynab_account_ids,
        external_account_ids: db_data_cloned.base.external_account_ids,
        balances: BTreeMap::new(),
    };
    body.name = Faker.fake();
    body.balances
        .entry(year)
        .or_default()
        .insert(month, Some((-1000000..1000000).fake()));

    let body_cloned = body.clone();
    let mut expected_resp = FinancialResourceYearly {
        base: BaseFinancialResource {
            id: body_cloned.id,
            name: body_cloned.name,
            resource_type: body_cloned.resource_type,
            ynab_account_ids: body_cloned.ynab_account_ids,
            external_account_ids: body_cloned.external_account_ids,
        },
        balances: BTreeMap::new(),
    };

    for (year, month, balance) in body.balances.iter().flat_map(|(&year, month_balances)| {
        month_balances
            .iter()
            .map(move |(month, &balance)| (year, *month, balance))
    }) {
        if let Some(balance) = balance {
            expected_resp.insert_balance(year, month, balance);
        }
    }

    check_update(
        pool,
        body,
        Some(db_data),
        StatusCode::OK,
        Some(expected_resp),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_id_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool).await;

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/resources/{}", Faker.fake::<u32>()))
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
    let context = TestContext::setup(pool).await;

    #[derive(Debug, Clone, Serialize, Dummy)]
    struct ReqBody {
        pub id: Uuid,
        pub name: String,
    }
    let body = Faker.fake::<ReqBody>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/resources/{}", Faker.fake::<Uuid>()))
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
                .method("PUT")
                .uri(&format!("/resources/{}", Faker.fake::<Uuid>()))
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

    let body = Faker.fake::<UpdateBody>();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/resources/{}", Faker.fake::<Uuid>()))
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}
