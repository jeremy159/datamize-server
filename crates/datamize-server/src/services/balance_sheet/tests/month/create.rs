use chrono::{Datelike, NaiveDate};
use datamize_domain::{FinancialResourceMonthly, Month, MonthNum, NetTotal, SaveMonth};
use fake::{faker::chrono::en::Date, Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;

use crate::services::{
    balance_sheet::tests::month::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

fn are_equal(a: &Month, b: &Month) {
    assert_eq!(a.month, b.month);
    assert_eq!(a.year, b.year);
    assert_eq!(a.net_assets.total, b.net_assets.total);
    assert_eq!(a.net_assets.balance_var, b.net_assets.balance_var);
    assert_eq!(a.net_assets.percent_var, b.net_assets.percent_var);
    assert_eq!(a.net_portfolio.total, b.net_assets.total);
    assert_eq!(a.net_portfolio.balance_var, b.net_portfolio.balance_var);
    assert_eq!(a.net_portfolio.percent_var, b.net_portfolio.percent_var);
    assert_eq!(a.resources, b.resources);
}

async fn check_create(
    pool: SqlitePool,
    year: Option<i32>,
    create_year: bool,
    new_month: SaveMonth,
    expected_resp: Option<Month>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool);

    if let Some(year) = year {
        if create_year {
            context.insert_year(year).await;
        }
    }
    let year = year.unwrap_or(Date().fake::<NaiveDate>().year());

    let response = context.service().create_month(year, new_month).await;

    if let Some(mut expected_resp) = expected_resp {
        expected_resp.compute_net_totals();
        let res_body = response.unwrap();
        are_equal(&res_body, &expected_resp);

        let saved = context
            .get_month(expected_resp.month, expected_resp.year)
            .await
            .unwrap();
        assert_eq!(res_body, saved);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn persists_new_month(pool: SqlitePool) {
    let body: SaveMonth = Faker.fake();
    let year = Date().fake::<NaiveDate>().year();
    let month: Month = Month {
        month: body.month,
        year,
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
        resources: vec![],
        ..Faker.fake()
    };

    check_create(pool, Some(year), true, body, Some(month), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_not_found_when_year_does_not_exist(pool: SqlitePool) {
    check_create(
        pool,
        None,
        false,
        Faker.fake(),
        None,
        Some(ErrorType::NotFound),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_error_already_exists_when_month_already_exists(pool: SqlitePool) {
    let body: SaveMonth = Faker.fake();
    let year = Date().fake::<NaiveDate>().year();
    {
        let context = TestContext::setup(pool.clone());
        context.insert_year(year).await;
        let month = Month {
            month: body.month,
            year,
            ..Faker.fake()
        };
        let month = Month {
            resources: month
                .resources
                .into_iter()
                .map(|r| FinancialResourceMonthly {
                    year,
                    month: month.month,
                    ..r
                })
                .collect(),
            ..month
        };

        context.set_month(&month, year).await;
    }
    check_create(
        pool,
        Some(year),
        false,
        body,
        None,
        Some(ErrorType::AlreadyExist),
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_if_prev_month_exists(pool: SqlitePool) {
    let month: i16 = (2..12).fake();
    let year = Date().fake::<NaiveDate>().year();
    let body = SaveMonth {
        month: month.try_into().unwrap(),
    };
    let prev_month = Month {
        month: (month - 1).try_into().unwrap(),
        year,
        ..Faker.fake()
    };
    let mut prev_month = Month {
        resources: prev_month
            .resources
            .into_iter()
            .map(|r| FinancialResourceMonthly {
                year: prev_month.year,
                month: prev_month.month,
                ..r
            })
            .collect(),
        ..prev_month
    };
    prev_month.compute_net_totals();

    let month = Month {
        month: body.month,
        year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_month.net_assets.total,
            percent_var: if prev_month.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_month.net_portfolio.total,
            percent_var: if prev_month.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        resources: vec![],
        ..Faker.fake()
    };

    {
        let context = TestContext::setup(pool.clone());
        context.insert_year(year).await;
        context.set_month(&prev_month, year).await;
    }
    check_create(pool, Some(year), false, body, Some(month), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn update_net_totals_if_prev_month_exists_in_prev_year(pool: SqlitePool) {
    let year = Date().fake::<NaiveDate>().year();
    let body = SaveMonth {
        month: MonthNum::January,
    };
    let prev_month = Month {
        month: MonthNum::December,
        year: year - 1,
        ..Faker.fake()
    };
    let mut prev_month = Month {
        resources: prev_month
            .resources
            .into_iter()
            .map(|r| FinancialResourceMonthly {
                year: prev_month.year,
                month: prev_month.month,
                ..r
            })
            .collect(),
        ..prev_month
    };
    prev_month.compute_net_totals();

    let month = Month {
        month: body.month,
        year,
        net_assets: NetTotal {
            total: 0,
            balance_var: -prev_month.net_assets.total,
            percent_var: if prev_month.net_assets.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        net_portfolio: NetTotal {
            total: 0,
            balance_var: -prev_month.net_portfolio.total,
            percent_var: if prev_month.net_portfolio.total == 0 {
                0.0
            } else {
                -1.0
            },
            ..Faker.fake()
        },
        resources: vec![],
        ..Faker.fake()
    };

    {
        let context = TestContext::setup(pool.clone());
        context.insert_year(year - 1).await;
        context.set_month(&prev_month, year - 1).await;
    }
    check_create(pool, Some(year), true, body, Some(month), None).await;
}
