use crate::{NetTotalType, Year};
use chrono::{Datelike, NaiveDate};
use fake::{faker::chrono::en::Date, Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};

use crate::NetTotal;

#[test]
fn sets_percent_var_as_0_when_no_prev_total() {
    let date = Date().fake::<NaiveDate>();
    let mut year = Year::new(date.year());
    let prev_net_total = NetTotal {
        total: 0,
        ..Faker.fake()
    };

    year.update_net_assets_with_previous(&prev_net_total);
    assert_eq!(year.net_assets.percent_var, 0.0);

    year.update_net_portfolio_with_previous(&prev_net_total);
    assert_eq!(year.net_portfolio.percent_var, 0.0);
}

#[test]
fn correctly_updates_balance_and_percent_var() {
    let date = Date().fake::<NaiveDate>();
    let mut year = Year::new(date.year());
    let prev_net_total = NetTotal {
        total: (1..1000000).fake(),
        ..Faker.fake()
    };

    let net_assets_before = year.net_assets.clone();
    let net_portfolio_before = year.net_portfolio.clone();

    year.update_net_assets_with_previous(&prev_net_total);
    assert_eq!(net_assets_before.total, year.net_assets.total); // Not affected
    assert_ne!(net_assets_before.balance_var, year.net_assets.balance_var);
    assert_ne!(net_assets_before.percent_var, year.net_assets.percent_var);

    year.update_net_portfolio_with_previous(&prev_net_total);
    assert_eq!(net_portfolio_before.total, year.net_portfolio.total); // Not affected
    assert_ne!(
        net_portfolio_before.balance_var,
        year.net_portfolio.balance_var
    );
    assert_ne!(
        net_portfolio_before.percent_var,
        year.net_portfolio.percent_var
    );
}

#[test]
fn correctly_updates_total() {
    let date = Date().fake::<NaiveDate>();
    let mut year = Year::new(date.year());
    let prev_net_total = NetTotal {
        total: (1..1000000).fake(),
        ..Faker.fake()
    };

    let net_assets_before = year.net_assets.clone();
    let net_portfolio_before = year.net_portfolio.clone();

    year.update_net_assets_with_last_month(&prev_net_total);
    assert_ne!(net_assets_before.total, year.net_assets.total);
    assert_eq!(net_assets_before.balance_var, year.net_assets.balance_var); // Not affected
    assert_eq!(net_assets_before.percent_var, year.net_assets.percent_var); // Not affected

    year.update_net_portfolio_with_last_month(&prev_net_total);
    assert_ne!(net_portfolio_before.total, year.net_portfolio.total);
    assert_eq!(
        net_portfolio_before.balance_var,
        year.net_portfolio.balance_var
    ); // Not affected
    assert_eq!(
        net_portfolio_before.percent_var,
        year.net_portfolio.percent_var
    ); // Not affected
}

#[test]
fn does_not_need_update_if_total_is_same() {
    let month_net_assets = NetTotal {
        net_type: NetTotalType::Asset,
        ..Faker.fake()
    };
    let month_net_portfolio = NetTotal {
        net_type: NetTotalType::Portfolio,
        ..Faker.fake()
    };
    let year = Year {
        net_assets: month_net_assets.clone(),
        net_portfolio: month_net_portfolio.clone(),
        ..Faker.fake()
    };

    assert!(!year.needs_net_totals_update(&month_net_assets, &month_net_portfolio));
}

#[test]
fn does_need_update_if_net_assets_total_is_not_same() {
    let month_net_assets = NetTotal {
        net_type: NetTotalType::Asset,
        ..Faker.fake()
    };
    let month_net_portfolio = NetTotal {
        net_type: NetTotalType::Portfolio,
        ..Faker.fake()
    };
    let year = Year {
        net_portfolio: month_net_portfolio.clone(),
        ..Faker.fake()
    };

    assert!(year.needs_net_totals_update(&month_net_assets, &month_net_portfolio));
}

#[test]
fn does_need_update_if_net_portfolio_total_is_not_same() {
    let month_net_assets = NetTotal {
        net_type: NetTotalType::Asset,
        ..Faker.fake()
    };
    let month_net_portfolio = NetTotal {
        net_type: NetTotalType::Portfolio,
        ..Faker.fake()
    };
    let year = Year {
        net_assets: month_net_assets.clone(),
        ..Faker.fake()
    };

    assert!(year.needs_net_totals_update(&month_net_assets, &month_net_portfolio));
}
