use std::collections::BTreeMap;

use datamize_domain::{BaseFinancialResource, FinancialResourceYearly, SaveResource};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;

use crate::services::balance_sheet::tests::financial_resource::testutils::{
    assert_err, ErrorType, TestContext,
};

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
    new_res: SaveResource,
    expected_resp: Option<FinancialResourceYearly>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);
    let year = expected_resp.clone().unwrap_or_else(|| Faker.fake()).year;

    if create_year {
        context.insert_year(year).await;
    }

    let response = context.service().create_fin_res(new_res).await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        are_equal(&res_body, &expected_resp);

        if !expected_resp.balance_per_month.is_empty() {
            // Persits the resource
            let saved = context
                .get_res_by_name(&expected_resp.base.name)
                .await
                .unwrap();
            assert!(!saved.is_empty());
            assert_eq!(res_body, saved[0]);

            // Creates all months that were not created
            for m in expected_resp.balance_per_month.keys() {
                let saved_month = context.get_month(*m, expected_resp.year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                // Since net_assets are computed from all resources' type
                assert_ne!(saved_month.net_assets.total, 0);
            }
        }

        // Creating the resource also computed net assets of the year
        let saved_year = context.get_year(expected_resp.year).await;
        assert!(saved_year.is_ok());
        let saved_year = saved_year.unwrap();
        assert_ne!(saved_year.net_assets.total, 0);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_resource(pool: SqlitePool) {
    let body: SaveResource = Faker.fake();
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

    check_create(pool, true, body, Some(res), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_year_does_not_exist(pool: SqlitePool) {
    check_create(pool, false, Faker.fake(), None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_already_exists_when_resource_already_exists(pool: SqlitePool) {
    let mut balance_per_month = BTreeMap::new();
    let month = Faker.fake();
    balance_per_month.insert(month, (-1000000..1000000).fake());
    let body = SaveResource {
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
    check_create(pool, false, body, None, Some(ErrorType::AlreadyExist)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn new_resource_updates_all_months(pool: SqlitePool) {
    let mut balance_per_month = BTreeMap::new();
    balance_per_month.insert(
        TryFrom::<i16>::try_from(1).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(2).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(3).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(7).unwrap(),
        (-1000000..1000000).fake(),
    );
    balance_per_month.insert(
        TryFrom::<i16>::try_from(11).unwrap(),
        (-1000000..1000000).fake(),
    );
    let body = SaveResource {
        balance_per_month,
        ..Faker.fake()
    };
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

    check_create(pool, true, body, Some(res), None).await;
}
