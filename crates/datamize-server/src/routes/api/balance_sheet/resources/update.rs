use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{BaseFinancialResource, FinancialResourceYearly, SaveResource};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::resources::testutils::{
    correctly_stub_resource, TestContext,
};

fn are_equal(a: &FinancialResourceYearly, b: &FinancialResourceYearly) {
    assert_eq!(a.year, b.year);
    assert_eq!(a.base.name, b.base.name);
    assert_eq!(a.base.category, b.base.category);
    assert_eq!(a.base.r_type, b.base.r_type);
    assert_eq!(a.base.editable, b.base.editable);
    assert_eq!(a.base.ynab_account_ids, b.base.ynab_account_ids);
    assert_eq!(a.base.external_account_ids, b.base.external_account_ids);
}

async fn check_update(
    pool: SqlitePool,
    create_year: bool,
    req_body: Option<SaveResource>,
    expected_status: StatusCode,
    expected_resp: Option<FinancialResourceYearly>,
) {
    let context = TestContext::setup(pool);

    let year = req_body.clone().expect("missing body to create year").year;
    if create_year {
        context.insert_year(year).await;

        // Create all months
        for m in expected_resp
            .clone()
            .unwrap_or_else(|| Faker.fake())
            .balance_per_month
            .keys()
        {
            context.insert_month(*m, year).await;
        }
    }

    let expected_resp = correctly_stub_resource(expected_resp, year);
    if let Some(expected_resp) = expected_resp.clone() {
        context.set_resources(&[expected_resp]).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/resources/{:?}",
                    expected_resp
                        .clone()
                        .unwrap_or_else(|| Faker.fake())
                        .base
                        .id
                ))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = expected_resp {
        let body: FinancialResourceYearly = serde_json::from_slice(&body).unwrap();
        are_equal(&body, &expected);
        if let Some(req_body) = req_body {
            // Make sure the requested body is not equal to the resource that was in the db. I.e. new balance per month should have updated something
            assert_ne!(
                Into::<FinancialResourceYearly>::into(req_body.clone()),
                expected
            );

            // Make sure the update is persisted in db
            let saved = context.get_res(expected.base.id).await.unwrap();
            are_equal(&req_body.clone().into(), &saved);

            let mut req_balance = req_body.clone().balance_per_month;
            let mut expected_balance = expected.clone().balance_per_month;
            expected_balance.append(&mut req_balance);
            assert_eq!(expected_balance, saved.balance_per_month);

            if !req_body.balance_per_month.is_empty() {
                // Creates all months that were not created
                for m in req_body.balance_per_month.keys() {
                    let saved_month = context.get_month(*m, expected.year).await;
                    assert!(saved_month.is_ok());

                    let saved_month = saved_month.unwrap();
                    // Since net_assets are computed from all resources' type
                    assert_ne!(saved_month.net_assets.total, 0);
                }
            }

            // Updating the resource also computed net assets of the year
            let saved_year = context.get_year(expected.year).await;
            assert!(saved_year.is_ok());
            let saved_year = saved_year.unwrap();
            assert_ne!(saved_year.net_assets.total, 0);
        }
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_no_year(pool: SqlitePool) {
    check_update(pool, false, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, true, Some(Faker.fake()), StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body: SaveResource = Faker.fake();
    let body_cloned = body.clone();
    let expected_resp = FinancialResourceYearly {
        year: body_cloned.year,
        base: BaseFinancialResource {
            name: body_cloned.name,
            category: body_cloned.category,
            r_type: body_cloned.r_type,
            editable: body_cloned.editable,
            ynab_account_ids: body_cloned.ynab_account_ids,
            external_account_ids: body_cloned.external_account_ids,
            ..Faker.fake()
        },
        ..Faker.fake()
    };

    check_update(pool, true, Some(body), StatusCode::OK, Some(expected_resp)).await;
}
