use std::collections::BTreeMap;

use datamize_domain::{BaseFinancialResource, FinancialResourceYearly, SaveResource};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::financial_resource::testutils::{correctly_stub_resource, TestContext},
    testutils::{assert_err, ErrorType},
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

async fn check_update(
    pool: SqlitePool,
    create_year: bool,
    updated_res: SaveResource,
    expected_resp: Option<FinancialResourceYearly>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);
    let year = updated_res.year;

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
        .service()
        .update_fin_res(
            expected_resp
                .clone()
                .unwrap_or_else(|| Faker.fake())
                .base
                .id,
            updated_res.clone(),
        )
        .await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        are_equal(&res_body, &expected_resp);

        // Make sure the requested body is not equal to the resource that was in the db. I.e. new balance per month should have updated something
        assert_ne!(
            Into::<FinancialResourceYearly>::into(updated_res.clone()),
            expected_resp
        );

        // Make sure the update is persisted in db
        let saved = context.get_res(expected_resp.base.id).await.unwrap();
        are_equal(&updated_res.clone().into(), &saved);

        let mut req_balance = updated_res.clone().balance_per_month;
        let mut expected_balance = expected_resp.clone().balance_per_month;
        expected_balance.append(&mut req_balance);
        assert_eq!(expected_balance, saved.balance_per_month);

        if !updated_res.balance_per_month.is_empty() {
            // Creates all months that were not created
            for m in updated_res.balance_per_month.keys() {
                let saved_month = context.get_month(*m, expected_resp.year).await;
                assert!(saved_month.is_ok());

                let saved_month = saved_month.unwrap();
                if !saved_month.resources.is_empty() {
                    // Since net_assets are computed from all resources' type
                    assert_ne!(saved_month.net_assets.total, 0);
                }
            }
        }

        // Updating the resource also computed net assets of the year
        let saved_year = context.get_year(expected_resp.year).await;
        assert!(saved_year.is_ok());
        let saved_year = saved_year.unwrap();
        if let Some(last_month) = saved_year.get_last_month() {
            assert_eq!(saved_year.net_assets.total, last_month.net_assets.total);
        }
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_no_year(pool: SqlitePool) {
    check_update(pool, false, Faker.fake(), None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_nothing_in_db(pool: SqlitePool) {
    check_update(pool, true, Faker.fake(), None, Some(ErrorType::NotFound)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_the_update(pool: SqlitePool) {
    let body: SaveResource = Faker.fake();
    let body_cloned = body.clone();
    let expected_resp = FinancialResourceYearly {
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

    check_update(pool, true, body, Some(expected_resp), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn updated_resource_updates_all_months(pool: SqlitePool) {
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

    check_update(pool, true, body, Some(res), None).await;
}
