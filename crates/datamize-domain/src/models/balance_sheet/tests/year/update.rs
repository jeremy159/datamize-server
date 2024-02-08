use crate::{Month, NetTotals, Year};
use chrono::{Datelike, NaiveDate};
use fake::{faker::chrono::en::Date, Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};

use crate::NetTotal;

#[test]
fn sets_percent_var_as_0_when_no_prev_total() {
    let date = Date().fake::<NaiveDate>();
    let mut year = Year::new(date.year());
    let prev_year = Year {
        net_totals: NetTotals {
            assets: NetTotal {
                total: 0,
                ..Faker.fake()
            },
            portfolio: NetTotal {
                total: 0,
                ..Faker.fake()
            },
        },
        ..Faker.fake()
    };

    year.compute_variation(&prev_year);
    assert_eq!(year.net_assets().percent_var, 0.0);

    assert_eq!(year.net_portfolio().percent_var, 0.0);
}

#[test]
fn correctly_updates_balance_and_percent_var() {
    let date = Date().fake::<NaiveDate>();
    let mut year = Year::new(date.year());
    let prev_year: Year = Faker.fake();

    let net_assets_before = year.net_assets().clone();
    let net_portfolio_before = year.net_portfolio().clone();

    year.compute_variation(&prev_year);

    assert_eq!(net_assets_before.total, year.net_assets().total); // Not affected
    assert_ne!(net_assets_before.balance_var, year.net_assets().balance_var);
    assert_ne!(net_assets_before.percent_var, year.net_assets().percent_var);

    assert_eq!(net_portfolio_before.total, year.net_portfolio().total); // Not affected
    assert_ne!(
        net_portfolio_before.balance_var,
        year.net_portfolio().balance_var
    );
    assert_ne!(
        net_portfolio_before.percent_var,
        year.net_portfolio().percent_var
    );
}

#[test]
fn correctly_updates_total() {
    let date = Date().fake::<NaiveDate>();
    let mut year = Year::new(date.year());
    year.months.push(Faker.fake());

    let net_assets_before = year.net_assets().clone();
    let net_portfolio_before = year.net_portfolio().clone();

    year.update_net_totals();

    assert_ne!(net_assets_before.total, year.net_assets().total);
    assert_eq!(net_assets_before.balance_var, year.net_assets().balance_var); // Not affected
    assert_eq!(net_assets_before.percent_var, year.net_assets().percent_var); // Not affected

    assert_ne!(net_portfolio_before.total, year.net_portfolio().total);
    assert_eq!(
        net_portfolio_before.balance_var,
        year.net_portfolio().balance_var
    ); // Not affected
    assert_eq!(
        net_portfolio_before.percent_var,
        year.net_portfolio().percent_var
    ); // Not affected
}

#[test]
fn does_not_need_update_if_total_is_same() {
    let month: Month = Faker.fake();
    let year = Year {
        net_totals: NetTotals {
            assets: month.net_assets().clone(),
            portfolio: month.net_portfolio().clone(),
        },
        ..Faker.fake()
    };

    assert!(!year.needs_update(&month));
}

#[test]
fn does_need_update_if_net_assets_total_is_not_same() {
    let month: Month = Faker.fake();
    let year = Year {
        net_totals: NetTotals {
            assets: Faker.fake(),
            portfolio: month.net_portfolio().clone(),
        },
        ..Faker.fake()
    };

    assert!(year.needs_update(&month));
}

#[test]
fn does_need_update_if_net_portfolio_total_is_not_same() {
    let month: Month = Faker.fake();
    let year = Year {
        net_totals: NetTotals {
            assets: month.net_assets().clone(),
            portfolio: Faker.fake(),
        },
        ..Faker.fake()
    };

    assert!(year.needs_update(&month));
}
