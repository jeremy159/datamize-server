use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::Year;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::years::testutils::{
    correctly_stub_year, transform_expected_year, TestContext,
};

async fn check_delete(pool: SqlitePool, expected_status: StatusCode, expected_resp: Option<Year>) {
    let context = TestContext::setup(pool);

    let expected_resp = correctly_stub_year(expected_resp);
    if let Some(expected_resp) = &expected_resp {
        context.set_year(expected_resp).await;
    }

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/years/{:?}",
                    expected_resp.clone().unwrap_or_else(|| Faker.fake()).year
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Some(expected) = transform_expected_year(expected_resp) {
        let body: Year = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);

        // Make sure the deletion removed it from db
        let saved = context.get_year(expected.year).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        // Make sure the deletion removed net totals of the year from db
        let saved_net_totals = context.get_net_totals(expected.id).await;
        assert_eq!(saved_net_totals, Ok(vec![]));

        // Make sure the deletion removed months of the year from db
        let saved_months = context.get_months(expected.year).await;
        assert_eq!(saved_months, Ok(vec![]));

        // Make sure the deletion removed saving rates of the year from db
        let saved_saving_rates = context.get_saving_rates(expected.year).await;
        assert_eq!(saved_saving_rates, Ok(vec![]));

        // Make sure the deletion removed resources of the year from db
        let saved_resources = context.get_resources(expected.year).await;
        assert_eq!(saved_resources, Ok(vec![]));
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, StatusCode::OK, Some(Faker.fake())).await;
}

// #[sqlx::test(migrations = "../db-sqlite/migrations")]
// async fn does_not_delete_same_month_of_different_year(pool: SqlitePool) {
//     let month: Month = Faker.fake();
//     let same_month_other_year = Month {
//         year: month.year + 1,
//         month: month.month,
//         ..Faker.fake()
//     };
//     let context = TestContext::setup(pool.clone());
//     context.insert_year(same_month_other_year.year).await;
//     let mut same_month_other_year = Month {
//         resources: same_month_other_year
//             .resources
//             .into_iter()
//             .map(|r| FinancialResourceMonthly {
//                 year: same_month_other_year.year,
//                 month: same_month_other_year.month,
//                 ..r
//             })
//             .collect(),
//         ..same_month_other_year
//     };
//     same_month_other_year
//         .resources
//         .sort_by(|a, b| a.base.name.cmp(&b.base.name));

//     context
//         .set_month(&same_month_other_year, same_month_other_year.year)
//         .await;

//     check_delete(pool, true, StatusCode::OK, Some(month)).await;
//     // Make sure the deletion did not remove the other month
//     let mut saved = context
//         .get_month(same_month_other_year.month, same_month_other_year.year)
//         .await
//         .unwrap();
//     saved
//         .resources
//         .sort_by(|a, b| a.base.name.cmp(&b.base.name));
//     assert_eq!(saved, same_month_other_year);
// }
