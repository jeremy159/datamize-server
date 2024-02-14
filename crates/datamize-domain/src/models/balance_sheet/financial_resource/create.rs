use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    BalancePerYearPerMonth, BaseFinancialResource, FinancialResourceType, FinancialResourceYearly,
};
use crate::YearlyBalances;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveResource {
    pub name: String,
    pub resource_type: FinancialResourceType,
    pub balances: BalancePerYearPerMonth,
    pub ynab_account_ids: Option<Vec<Uuid>>,
    pub external_account_ids: Option<Vec<Uuid>>,
}

impl YearlyBalances for SaveResource {
    fn balances(&self) -> &BalancePerYearPerMonth {
        &self.balances
    }

    fn balances_mut(&mut self) -> &mut BalancePerYearPerMonth {
        &mut self.balances
    }
}

impl From<SaveResource> for FinancialResourceYearly {
    fn from(value: SaveResource) -> Self {
        FinancialResourceYearly {
            base: BaseFinancialResource::new(
                value.name,
                value.resource_type,
                value.ynab_account_ids,
                value.external_account_ids,
            ),
            balances: value.balances,
        }
    }
}

#[cfg(any(feature = "testutils", test))]
impl fake::Dummy<fake::Faker> for SaveResource {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        use crate::testutils::NUM_MONTHS;
        use fake::{Fake, Faker};
        use std::collections::BTreeMap;

        let name = Fake::fake_with_rng(&Faker, rng);
        let resource_type = Fake::fake_with_rng(&Faker, rng);

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
        let ynab_account_ids = Fake::fake_with_rng(&Faker, rng);
        let external_account_ids = Fake::fake_with_rng(&Faker, rng);

        Self {
            name,
            resource_type,
            balances,
            ynab_account_ids,
            external_account_ids,
        }
    }
}
