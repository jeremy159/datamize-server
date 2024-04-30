use chrono::{Datelike, NaiveDate};
use datamize_domain::{
    testutils::financial_resource_yearly_equal_without_id, FinancialResourceYearly, SaveResource,
    YearlyBalances,
};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

async fn check_create(
    pool: SqlitePool,
    new_res: SaveResource,
    expected_resp: Option<FinancialResourceYearly>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool).await;

    let response = context.service().create_fin_res(new_res).await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        financial_resource_yearly_equal_without_id(&res_body, &expected_resp);

        if !expected_resp.is_empty() {
            // Persits the resource
            let saved = context
                .get_res_by_name(&expected_resp.base.name)
                .await
                .unwrap();
            assert!(!saved.is_empty());
            assert_eq!(res_body, saved);

            // Creates all months that were not created
            for (year, month, _) in expected_resp.iter_balances() {
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
    } else {
        println!("{response:#?}");
        assert_err(response.unwrap_err(), expected_err);
    }
}

// Previous behavoir was to return 404 when year did not exist, but now, just like month
// we create it automatically. This test is to ensure we have the right behavior.
#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_resource(pool: SqlitePool) {
    let mut body: SaveResource = Faker.fake();
    body.clear_all_balances();
    let current_date = Faker.fake::<NaiveDate>();
    let month = current_date.month().try_into().unwrap();
    let year = current_date.year();
    body.insert_balance(year, month, (-1000000..1000000).fake());

    check_create(pool, body.clone(), Some(body.into()), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_already_exists_when_resource_already_exists(pool: SqlitePool) {
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

    let body = SaveResource {
        name: resource.base.name.clone(),
        ..Faker.fake()
    };
    let context = TestContext::setup(pool.clone()).await;
    context.insert_year(year).await;
    context.insert_month(month, year).await;
    context.set_resources(&[resource]).await;

    check_create(pool, body, None, Some(ErrorType::AlreadyExist)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn new_resource_updates_all_months_and_all_years(pool: SqlitePool) {
    let mut res = FinancialResourceYearly::new(
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
        Faker.fake(),
    );
    let years: [i32; 2] = [(1000..3000).fake(), (1000..3000).fake()];
    let months: [i16; 5] = [1, 2, 3, 8, 11];
    for month in months {
        let idx = (month % 2) as usize;
        let year = years[idx];
        res.insert_balance(year, month.try_into().unwrap(), (-1000000..1000000).fake());
    }
    let res_cloned = res.clone();
    let body = SaveResource {
        name: res_cloned.base.name,
        resource_type: res_cloned.base.resource_type,
        balances: res_cloned.balances,
        ynab_account_ids: res_cloned.base.ynab_account_ids,
        external_account_ids: res_cloned.base.external_account_ids,
    };

    check_create(pool, body, Some(res), None).await;
}
