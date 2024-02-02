use datamize_domain::Year;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::year::testutils::{
        correctly_stub_year, transform_expected_year, TestContext,
    },
    testutils::{assert_err, ErrorType},
};

async fn check_delete(
    pool: SqlitePool,
    expected_resp: Option<Year>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    let expected_resp = correctly_stub_year(expected_resp);
    if let Some(expected_resp) = &expected_resp {
        context.set_year(expected_resp).await;
    }

    let response = context
        .service()
        .delete_year(expected_resp.clone().unwrap_or_else(|| Faker.fake()).year)
        .await;
    let expected_resp = transform_expected_year(expected_resp);

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        assert_eq!(res_body, expected_resp);

        // Make sure the deletion removed it from db
        let saved = context.get_year(expected_resp.year).await;
        assert_eq!(saved, Err(datamize_domain::db::DbError::NotFound));

        // Make sure the deletion removed net totals of the year from db
        let saved_net_totals = context.get_net_totals(expected_resp.id).await;
        assert_eq!(saved_net_totals, Ok(vec![]));

        // Make sure the deletion removed months of the year from db
        let saved_months = context.get_months(expected_resp.year).await;
        assert_eq!(saved_months, Ok(vec![]));

        // Make sure the deletion removed saving rates of the year from db
        let saved_saving_rates = context.get_saving_rates(expected_resp.year).await;
        assert_eq!(saved_saving_rates, Ok(vec![]));

        // Make sure the deletion removed resources of the year from db
        let saved_resources = context.get_resources(expected_resp.year).await;
        assert_eq!(saved_resources, Ok(vec![]));
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_delete(pool, None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_deletion(pool: SqlitePool) {
    check_delete(pool, Some(Faker.fake()), None).await;
}
