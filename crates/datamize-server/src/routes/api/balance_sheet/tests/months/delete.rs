use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use datamize_domain::{net_totals_equal_without_id, Month, MonthNum, NetTotals, Uuid};
use fake::{Fake, Faker};
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::routes::api::balance_sheet::tests::months::testutils::{
    transform_expected_month, TestContext,
};

async fn check_delete(
    pool: SqlitePool,
    create_year: bool,
    expected_status: StatusCode,
    expected_resp: Option<Month>,
) {
    let context = TestContext::setup(pool);

    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;
    if create_year {
        context.insert_year(year).await;
    }

    if let Some(expected_resp) = expected_resp.clone() {
        context.set_month(&expected_resp, year).await;
    }
    let month: i16 = expected_resp
        .clone()
        .unwrap_or_else(|| Faker.fake())
        .month
        .into();

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/years/{:?}/months/{:?}", year, month))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    if let Some(expected) = transform_expected_month(expected_resp) {
        let body: Month = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected);

        // Make sure the deletion removed it from db
        let saved = context.get_month(expected.month, expected.year).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        // Make sure the deletion removed net totals of the month from db
        let saved_net_totals = context.get_net_totals(expected.id).await.unwrap();
        assert!(net_totals_equal_without_id(
            &saved_net_totals,
            &NetTotals::default()
        ));
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_no_year(pool: SqlitePool) {
    check_delete(pool, false, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_404_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, true, StatusCode::NOT_FOUND, None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, true, StatusCode::OK, Some(Faker.fake())).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn does_not_delete_same_month_of_different_year(pool: SqlitePool) {
    let month: Month = Faker.fake();
    let mut same_month_other_year = Month {
        year: month.year + 1,
        month: month.month,
        ..Faker.fake()
    };
    let context = TestContext::setup(pool.clone());
    context.insert_year(same_month_other_year.year).await;
    same_month_other_year
        .resources
        .sort_by(|a, b| a.base.name.cmp(&b.base.name));

    context
        .set_month(&same_month_other_year, same_month_other_year.year)
        .await;

    check_delete(pool, true, StatusCode::OK, Some(month)).await;
    // Make sure the deletion did not remove the other month
    let mut saved = context
        .get_month(same_month_other_year.month, same_month_other_year.year)
        .await
        .unwrap();
    saved
        .resources
        .sort_by(|a, b| a.base.name.cmp(&b.base.name));
    assert_eq!(saved, same_month_other_year);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_year_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/years/{}/months/{:?}",
                    Faker.fake::<Uuid>(),
                    Faker.fake::<MonthNum>()
                ))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_400_for_invalid_month_in_path(pool: SqlitePool) {
    let context = TestContext::setup(pool);

    let response = context
        .app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!(
                    "/years/{}/months/{:?}",
                    Faker.fake::<i32>(),
                    Faker.fake::<Uuid>()
                ))
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
