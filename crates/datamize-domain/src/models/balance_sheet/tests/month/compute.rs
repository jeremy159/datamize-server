use crate::{
    AssetType, BaseFinancialResource, FinancialResourceMonthly, FinancialResourceType, Month,
};
use chrono::{Datelike, NaiveDate};
use fake::{faker::chrono::en::Date, Fake, Faker};
use pretty_assertions::assert_eq;

#[test]
fn sets_total_at_0_when_no_resources() {
    let date = Date().fake::<NaiveDate>();
    let mut month = Month::new(date.month().try_into().unwrap(), date.year());

    month.compute_net_totals();
    assert_eq!(month.net_assets().total, 0);
    assert_eq!(month.net_portfolio().total, 0);
}

#[test]
fn does_not_overwrite_prev_total_when_no_resources() {
    let mut month = Month {
        resources: vec![],
        ..Faker.fake()
    };

    let net_assets_before = month.net_assets().clone();
    let net_portfolio_before = month.net_portfolio().clone();

    month.compute_net_totals();
    assert_eq!(net_assets_before.total, month.net_assets().total);
    assert_eq!(net_portfolio_before.total, month.net_portfolio().total);
}

#[test]
fn updates_total_when_at_least_1_resource() {
    let mut month = Month {
        resources: vec![FinancialResourceMonthly {
            base: BaseFinancialResource {
                resource_type: FinancialResourceType::Asset(AssetType::Cash),
                ..Faker.fake()
            },
            ..Faker.fake()
        }],
        ..Faker.fake()
    };

    let net_assets_before = month.net_assets().clone();
    let net_portfolio_before = month.net_portfolio().clone();

    month.compute_net_totals();
    assert_ne!(net_assets_before.total, month.net_assets().total);
    assert_ne!(net_portfolio_before.total, month.net_portfolio().total);
}

#[test]
fn equals_total_of_1_asset_resource() {
    let resource = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Asset(AssetType::Investment),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut month = Month {
        resources: vec![resource.clone()],
        ..Faker.fake()
    };

    month.compute_net_totals();
    assert_eq!(resource.balance, month.net_assets().total);
    assert_eq!(resource.balance, month.net_portfolio().total);
}

#[test]
fn equals_total_of_1_liability_resource() {
    let resource = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Liability(Faker.fake()),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut month = Month {
        resources: vec![resource.clone()],
        ..Faker.fake()
    };
    let net_portfolio_before = month.net_portfolio().clone();

    month.compute_net_totals();
    assert_eq!(-resource.balance, month.net_assets().total);
    assert_eq!(net_portfolio_before.total, month.net_portfolio().total);
}

#[test]
fn assets_equals_total_of_asset_minus_liability() {
    let res_asset = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Asset(AssetType::Investment),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let res_liability = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Liability(Faker.fake()),
            ..res_asset.base.clone()
        },
        ..res_asset.clone()
    };
    let mut month = Month {
        resources: vec![res_asset.clone(), res_liability.clone()],
        ..Faker.fake()
    };

    month.compute_net_totals();
    assert_eq!(0, month.net_assets().total);
    assert_eq!(res_asset.balance, month.net_portfolio().total);
}

#[test]
fn portfolio_equals_total_of_all_asset_except_long_term() {
    let res1_asset = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Asset(AssetType::Cash),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let res2_asset = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Asset(AssetType::Investment),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let res3_asset = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Asset(AssetType::LongTerm),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let res_liability = FinancialResourceMonthly {
        base: BaseFinancialResource {
            resource_type: FinancialResourceType::Liability(Faker.fake()),
            ..Faker.fake()
        },
        ..Faker.fake()
    };
    let mut month = Month {
        resources: vec![
            res1_asset.clone(),
            res2_asset.clone(),
            res3_asset.clone(),
            res_liability,
        ],
        ..Faker.fake()
    };

    month.compute_net_totals();
    assert_eq!(
        res1_asset.balance + res2_asset.balance,
        month.net_portfolio().total
    );
}
