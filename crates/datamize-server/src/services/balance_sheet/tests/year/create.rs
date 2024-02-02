use datamize_domain::{NetTotal, NetTotalType, SaveYear, Year};
use fake::{Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::year::testutils::{correctly_stub_year, TestContext},
    testutils::{assert_err, ErrorType},
};
fn are_equal(a: &Year, b: &Year) {
    assert_eq!(a.year, b.year);
    assert_eq!(a.net_assets.total, b.net_assets.total);
    assert_eq!(a.net_assets.balance_var, b.net_assets.balance_var);
    assert_eq!(a.net_assets.percent_var, b.net_assets.percent_var);
    assert_eq!(a.net_portfolio.total, b.net_assets.total);
    assert_eq!(a.net_portfolio.balance_var, b.net_portfolio.balance_var);
    assert_eq!(a.net_portfolio.percent_var, b.net_portfolio.percent_var);
    assert_eq!(a.months, b.months);
}

async fn check_create(
    pool: SqlitePool,
    new_year: SaveYear,
    expected_resp: Option<Year>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    let response = context.service().create_year(new_year).await;

    if let Some(expected_resp) = expected_resp {
        let res_body = response.unwrap();
        are_equal(&res_body, &expected_resp);

        let saved = context.get_year(expected_resp.year).await.unwrap();
        assert_eq!(res_body, saved);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_year(pool: SqlitePool) {
    let body: SaveYear = Faker.fake();
    let year = Year {
        year: body.year,
        net_assets: NetTotal {
            total: 0,
            balance_var: 0,
            percent_var: 0.0,
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: 0,
            percent_var: 0.0,
            ..Faker.fake()
        },
        months: vec![],
        ..Faker.fake()
    };

    check_create(pool, body.clone(), Some(year), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_already_exists_when_year_already_exists(pool: SqlitePool) {
    let body: SaveYear = Faker.fake();
    {
        let context = TestContext::setup(pool.clone());
        let year = Year {
            year: body.year,
            ..Faker.fake()
        };
        let year = correctly_stub_year(Some(year)).unwrap();
        context.set_year(&year).await;
    }
    check_create(pool, body, None, Some(ErrorType::AlreadyExist)).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_if_prev_year_exists(pool: SqlitePool) {
    let body: SaveYear = Faker.fake();
    let prev_year = Year {
        year: body.year - 1,
        ..Faker.fake()
    };
    let prev_year = correctly_stub_year(Some(prev_year)).unwrap();

    let year = Year {
        year: body.year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_year.net_assets.total,
            percent_var: if prev_year.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_year.net_portfolio.total,
            percent_var: if prev_year.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        months: vec![],
        ..Faker.fake()
    };

    {
        let context = TestContext::setup(pool.clone());
        context.set_year(&prev_year).await;
    }
    check_create(pool, body, Some(year), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_of_prev_and_next_years_if_exists(pool: SqlitePool) {
    let body: SaveYear = Faker.fake();
    let prev_year = Year {
        year: body.year - 1,
        ..Faker.fake()
    };
    let prev_year = correctly_stub_year(Some(prev_year)).unwrap();

    let next_year = Year {
        year: body.year + 1,
        ..Faker.fake()
    };
    let next_year = correctly_stub_year(Some(next_year)).unwrap();

    let year = Year {
        year: body.year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_year.net_assets.total,
            percent_var: if prev_year.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_year.net_portfolio.total,
            percent_var: if prev_year.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        months: vec![],
        ..Faker.fake()
    };

    let context = TestContext::setup(pool.clone());
    context.set_year(&prev_year).await;
    let next_year_id = context.set_year(&next_year).await;

    check_create(pool, body, Some(year), None).await;

    let saved_next_net_totals = context.get_net_totals(next_year_id).await.unwrap();
    for next_nt in saved_next_net_totals {
        match next_nt.net_type {
            NetTotalType::Asset => {
                assert_ne!(next_nt.balance_var, next_year.net_assets.balance_var);
                assert_ne!(next_nt.percent_var, next_year.net_assets.percent_var);
            }
            NetTotalType::Portfolio => {
                assert_ne!(next_nt.balance_var, next_year.net_portfolio.balance_var);
                assert_ne!(next_nt.percent_var, next_year.net_portfolio.percent_var);
            }
        };
    }
}
