use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{BalancePerYearPerMonth, BaseFinancialResource, FinancialResourceType};
use crate::YearlyBalances;

/// A resource represented within a year. It has a BTreeMap of balance per months.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FinancialResourceYearly {
    #[serde(flatten)]
    pub base: BaseFinancialResource,
    /// Balances per year with each year having a possibility of 12 balances (one for each month).
    /// This struct should not be manipulated manually but with the methods provided.
    pub balances: BalancePerYearPerMonth,
}

impl YearlyBalances for FinancialResourceYearly {
    fn balances(&self) -> &BalancePerYearPerMonth {
        &self.balances
    }

    fn balances_mut(&mut self) -> &mut BalancePerYearPerMonth {
        &mut self.balances
    }
}

impl FinancialResourceYearly {
    pub fn new(
        id: Uuid,
        name: String,
        resource_type: FinancialResourceType,
        ynab_account_ids: Option<Vec<Uuid>>,
        external_account_ids: Option<Vec<Uuid>>,
    ) -> Self {
        Self {
            base: BaseFinancialResource::new(
                name,
                resource_type,
                ynab_account_ids,
                external_account_ids,
            )
            .with_id(id),
            balances: BTreeMap::new(),
        }
    }
}
