use std::collections::BTreeMap;

use fake::{Fake, Faker};
use rand::seq::SliceRandom;

use crate::{FinancialResourceYearly, YearlyBalances};

pub const NUM_MONTHS: usize = 12;

#[cfg(any(feature = "testutils", test))]
impl fake::Dummy<fake::Faker> for FinancialResourceYearly {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        let base = Fake::fake_with_rng(&Faker, rng);
        let mut balances = BTreeMap::new();
        let len = (1..10).fake_with_rng(rng);
        for _ in 0..len {
            let len_values = (1..NUM_MONTHS).fake_with_rng(rng);
            let mut month_balances = BTreeMap::new();
            for _ in 0..len_values {
                let month = Fake::fake_with_rng(&Faker, rng);
                month_balances.insert(month, Some(Fake::fake_with_rng(&(-1000000..1000000), rng)));
            }
            balances.insert(Fake::fake_with_rng(&(1000..3000), rng), month_balances);
        }

        Self { base, balances }
    }
}

#[cfg(any(feature = "testutils", test))]
pub fn financial_resource_yearly_equal_without_id(
    a: &FinancialResourceYearly,
    b: &FinancialResourceYearly,
) -> bool {
    a.balances == b.balances
        && a.base.name == b.base.name
        && a.base.resource_type == b.base.resource_type
        && a.base.ynab_account_ids == b.base.ynab_account_ids
        && a.base.external_account_ids == b.base.external_account_ids
}

/// Will make sure the resources have the appropriate date associated to them
pub fn correctly_stub_resources(
    resources: Option<Vec<FinancialResourceYearly>>,
    years: [i32; 2],
) -> Option<Vec<FinancialResourceYearly>> {
    resources.map(|resources| {
        resources
            .into_iter()
            .map(|r| {
                let year = *years.choose(&mut rand::thread_rng()).unwrap();
                let mut res = FinancialResourceYearly::new(
                    r.base.id,
                    r.base.name.clone(),
                    r.base.resource_type.clone(),
                    r.base.ynab_account_ids.clone(),
                    r.base.external_account_ids.clone(),
                );
                for (_, month, balance) in r.iter_balances() {
                    res.insert_balance(year, month, balance);
                }

                res
            })
            .collect()
    })
}

/// Will make sure the resource has the appropriate date associated to it
pub fn correctly_stub_resource(
    resource: Option<FinancialResourceYearly>,
    year: i32,
) -> Option<FinancialResourceYearly> {
    resource.map(|r| {
        let mut res = FinancialResourceYearly::new(
            r.base.id,
            r.base.name.clone(),
            r.base.resource_type.clone(),
            r.base.ynab_account_ids.clone(),
            r.base.external_account_ids.clone(),
        );
        for (_, month, balance) in r.iter_balances() {
            res.insert_balance(year, month, balance);
        }

        res
    })
}

/// Will transform the expected response. In this case, resources should be sorted by year and then by name.
/// Will also filter out resources that don't have any balance in any month
pub fn transform_expected_resources(
    expected: Option<Vec<FinancialResourceYearly>>,
) -> Option<Vec<FinancialResourceYearly>> {
    expected.map(|mut expected| {
        // Filter resources without any balances
        expected.retain(|r| !r.is_empty());
        // Answer should be sorted by years and then by names
        expected.sort_by(|a, b| a.base.name.cmp(&b.base.name));
        expected
    })
}
