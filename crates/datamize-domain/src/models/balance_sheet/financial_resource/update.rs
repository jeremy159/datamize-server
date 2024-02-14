use serde::{Deserialize, Serialize};

use crate::{
    BalancePerYearPerMonth, BaseFinancialResource, FinancialResourceYearly, YearlyBalances,
};

/// To update a balance, send month_num: Some(balance)
/// but to delete a balance, send month_num: None. This differs from an abstend month which
/// just does not update anything on the month.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateResource {
    #[serde(flatten)]
    pub base: BaseFinancialResource,
    pub balances: BalancePerYearPerMonth,
}

impl From<UpdateResource> for FinancialResourceYearly {
    fn from(value: UpdateResource) -> Self {
        FinancialResourceYearly {
            base: value.base.clone(),
            balances: value.balances,
        }
    }
}

impl YearlyBalances for UpdateResource {
    fn balances(&self) -> &BalancePerYearPerMonth {
        &self.balances
    }

    fn balances_mut(&mut self) -> &mut BalancePerYearPerMonth {
        &mut self.balances
    }
}

#[cfg(any(feature = "testutils", test))]
impl fake::Dummy<fake::Faker> for UpdateResource {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        use crate::testutils::NUM_MONTHS;
        use fake::{Fake, Faker};
        use std::collections::BTreeMap;

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
