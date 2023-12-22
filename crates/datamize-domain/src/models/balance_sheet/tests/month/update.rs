use crate::Month;
use chrono::{Datelike, NaiveDate};
use fake::{faker::chrono::en::Date, Fake, Faker};
use pretty_assertions::{assert_eq, assert_ne};

use crate::NetTotal;

#[test]
fn sets_percent_var_as_0_when_no_prev_total() {
    let date = Date().fake::<NaiveDate>();
    let mut month = Month::new(date.month().try_into().unwrap(), date.year());
    let prev_net_total = NetTotal {
        total: 0,
        ..Faker.fake()
    };

    month.update_net_assets_with_previous(&prev_net_total);
    assert_eq!(month.net_assets.percent_var, 0.0);

    month.update_net_portfolio_with_previous(&prev_net_total);
    assert_eq!(month.net_portfolio.percent_var, 0.0);
}

#[test]
fn correctly_updates_balance_and_percent_var() {
    let date = Date().fake::<NaiveDate>();
    let mut month = Month::new(date.month().try_into().unwrap(), date.year());
    let prev_net_total = NetTotal {
        total: (1..1000000).fake(),
        ..Faker.fake()
    };

    let net_assets_before = month.net_assets.clone();
    let net_portfolio_before = month.net_portfolio.clone();

    month.update_net_assets_with_previous(&prev_net_total);
    assert_eq!(net_assets_before.total, month.net_assets.total); // Not affected
    assert_ne!(net_assets_before.balance_var, month.net_assets.balance_var);
    assert_ne!(net_assets_before.percent_var, month.net_assets.percent_var);

    month.update_net_portfolio_with_previous(&prev_net_total);
    assert_eq!(net_portfolio_before.total, month.net_portfolio.total); // Not affected
    assert_ne!(
        net_portfolio_before.balance_var,
        month.net_portfolio.balance_var
    );
    assert_ne!(
        net_portfolio_before.percent_var,
        month.net_portfolio.percent_var
    );
}
