use std::collections::BTreeMap;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{
    BaseFinancialResource, FinancialResourceYearly, MonthNum, ResourceCategory, ResourceType, Uuid,
};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::resources::testutils::TestContext;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CreateBody {
    pub name: String,
    pub category: ResourceCategory,
    #[serde(rename = "type")]
    pub r_type: ResourceType,
    pub editable: bool,
    pub year: i32,
    pub balance_per_month: BTreeMap<MonthNum, i64>,
    pub ynab_account_ids: Option<Vec<Uuid>>,
    pub external_account_ids: Option<Vec<Uuid>>,
}

impl fake::Dummy<fake::Faker> for CreateBody {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &fake::Faker, rng: &mut R) -> Self {
        let name = Fake::fake_with_rng(&Faker, rng);
        let category = Fake::fake_with_rng(&Faker, rng);
        let r_type = Fake::fake_with_rng(&Faker, rng);
        let editable = Fake::fake_with_rng(&Faker, rng);
        let year = Fake::fake_with_rng(&(1000..3000), rng);
        let ynab_account_ids = Fake::fake_with_rng(&Faker, rng);
        let external_account_ids = Fake::fake_with_rng(&Faker, rng);

        let mut balance_per_month = BTreeMap::new();
        let len = (1..10).fake_with_rng(rng);
        for _ in 0..len {
            balance_per_month.insert(
                config.fake_with_rng(rng),
                Fake::fake_with_rng(&(-1000000..1000000), rng),
            );
        }

        Self {
            name,
            category,
            r_type,
            editable,
            year,
            ynab_account_ids,
            external_account_ids,
            balance_per_month,
        }
    }
}

fn are_equal(a: &FinancialResourceYearly, b: &FinancialResourceYearly) {
    assert_eq!(a.year, b.year);
    assert_eq!(a.balance_per_month, b.balance_per_month);
    assert_eq!(a.base.name, b.base.name);
    assert_eq!(a.base.category, b.base.category);
    assert_eq!(a.base.r_type, b.base.r_type);
    assert_eq!(a.base.editable, b.base.editable);
    assert_eq!(a.base.ynab_account_ids, b.base.ynab_account_ids);
    assert_eq!(a.base.external_account_ids, b.base.external_account_ids);
}

async fn check_create(
    pool: SqlitePool,
    create_year: bool,
    body: Option<CreateBody>,
    expected_status: StatusCode,
    expected_resp: Option<FinancialResourceYearly>,
) {
    let context = TestContext::setup(pool);
    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;

    if create_year {
        context.insert_year(year).await;
    }

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

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        are_equal(&body, &expected);

        if !expected.balance_per_month.is_empty() {
            // Persits the resource
            let saved = context.get_res_by_name(&expected.base.name).await.unwrap();
            assert!(!saved.is_empty());
            assert_eq!(body, saved[0]);

            // Creates all months that were not created
            for m in expected.balance_per_month.keys() {
                let saved_month = context.get_month(*m, expected.year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                // Since net_assets are computed from all resources' type
                assert_ne!(saved_month.net_assets.total, 0);
            }
        }

        // Creating the resource also computed net assets of the year
        let saved_year = context.get_year(expected.year).await;
        assert!(saved_year.is_ok());
        let saved_year = saved_year.unwrap();
        assert_ne!(saved_year.net_assets.total, 0);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_resource(pool: SqlitePool) {
    let body: CreateBody = Faker.fake();
    let body_cloned = body.clone();
    let res = FinancialResourceYearly {
        year: body_cloned.year,
        balance_per_month: body_cloned.balance_per_month,
        base: BaseFinancialResource {
            name: body_cloned.name,
            category: body_cloned.category,
            r_type: body_cloned.r_type,
            editable: body_cloned.editable,
            ynab_account_ids: body_cloned.ynab_account_ids,
            external_account_ids: body_cloned.external_account_ids,
            ..Faker.fake()
        },
    };

    check_create(pool, true, Some(body), StatusCode::CREATED, Some(res)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_year_does_not_exist(pool: SqlitePool) {
    check_create(pool, false, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_409_when_resource_already_exists(pool: SqlitePool) {
    let mut balance_per_month = BTreeMap::new();
    let month = Faker.fake();
    balance_per_month.insert(month, (-1000000..1000000).fake());
    let body: CreateBody = CreateBody {
        balance_per_month,
        ..Faker.fake()
    };
    {
        let context = TestContext::setup(pool.clone());
        context.insert_year(body.year).await;
        context.insert_month(month, body.year).await;
        let body_cloned = body.clone();
        let res = FinancialResourceYearly {
            year: body_cloned.year,
            balance_per_month: body_cloned.balance_per_month,
            base: BaseFinancialResource {
                name: body_cloned.name,
                category: body_cloned.category,
                r_type: body_cloned.r_type,
                editable: body_cloned.editable,
                ynab_account_ids: body_cloned.ynab_account_ids,
                external_account_ids: body_cloned.external_account_ids,
                ..Faker.fake()
            },
        };

        context.set_resources(&[res]).await;
    }
    check_create(pool, false, Some(body), StatusCode::CONFLICT, None).await;
}
