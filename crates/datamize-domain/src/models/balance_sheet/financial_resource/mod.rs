mod create;
mod monthly;
mod res_type;
#[cfg(any(feature = "testutils", test))]
pub mod testutils;
mod update;
mod yearly;

pub use create::*;
pub use monthly::*;
pub use res_type::*;
pub use update::*;
pub use yearly::*;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::MonthNum;

pub type BalancePerMonth = BTreeMap<MonthNum, Option<i64>>;
pub type BalancePerYearPerMonth = BTreeMap<i32, BalancePerMonth>;

/// A resource with economic value. It represents either an asset or a liability
/// and adds more data to it.
#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct BaseFinancialResource {
    /// ID of the resource to be used when an update is needed.
    pub id: Uuid,
    /// The name of the resource.
    pub name: String,
    /// The type separates the resource in 2 groups: Assets vs Liabilities.
    /// Liabilities should have a negative balance.
    #[serde(with = "string")]
    pub resource_type: FinancialResourceType,
    /// Any YNAB accounts that should be used to refresh this resource's balance.
    pub ynab_account_ids: Option<Vec<Uuid>>,
    /// Any external accounts that should be used to refresh this resource's balance.
    /// They typically require a scrapping method in the `web_scraper` module.
    pub external_account_ids: Option<Vec<Uuid>>,
}

impl BaseFinancialResource {
    pub fn new(
        name: String,
        resource_type: FinancialResourceType,
        ynab_account_ids: Option<Vec<Uuid>>,
        external_account_ids: Option<Vec<Uuid>>,
    ) -> Self {
        BaseFinancialResource {
            id: Uuid::new_v4(),
            name,
            resource_type,
            ynab_account_ids,
            external_account_ids,
        }
    }

    pub fn with_id(self, id: Uuid) -> Self {
        BaseFinancialResource { id, ..self }
    }

    /// The resource can be sync with either ynab or external accounts
    pub fn syncable(&self) -> bool {
        self.ynab_account_ids.is_some() || self.external_account_ids.is_some()
    }
}

pub trait YearlyBalances {
    fn balances(&self) -> &BalancePerYearPerMonth;
    fn balances_mut(&mut self) -> &mut BalancePerYearPerMonth;

    fn insert_balance(&mut self, year: i32, month: MonthNum, balance: i64) {
        let year_entry = self.balances_mut().entry(year).or_default();

        year_entry.insert(month, Some(balance));
    }

    fn insert_balance_opt(&mut self, year: i32, month: MonthNum, balance: Option<i64>) {
        let year_entry = self.balances_mut().entry(year).or_default();

        year_entry.insert(month, balance);
    }

    fn insert_balance_for_year(&mut self, year: i32, balance: BalancePerMonth) {
        self.balances_mut().insert(year, balance);
    }

    /// Returns an iterator that allows extracting all months with balance, in all years
    fn iter_balances(&self) -> impl Iterator<Item = (i32, MonthNum, i64)> {
        self.balances().iter().flat_map(|(&year, month_balances)| {
            month_balances
                .iter()
                .filter_map(move |(month, &balance)| balance.map(|b| (year, *month, b)))
        })
    }

    /// Returns an iterator that allows extracting all months in all years even if they don't have balance.
    fn iter_all_balances(&self) -> impl Iterator<Item = (i32, MonthNum, Option<i64>)> {
        self.balances().iter().flat_map(|(&year, month_balances)| {
            month_balances
                .iter()
                .map(move |(month, &balance)| (year, *month, balance))
        })
    }

    /// Returns an iterator on all years
    fn iter_years(&self) -> impl Iterator<Item = i32> + '_ {
        self.balances().keys().copied()
    }

    /// Returns an iterator on all months (with their associated year) only if they have a balance
    fn iter_months(&self) -> impl Iterator<Item = (i32, MonthNum)> + '_ {
        self.balances().iter().flat_map(|(&year, month_balances)| {
            month_balances
                .iter()
                .filter_map(move |(month, &balance)| balance.map(|_| (year, *month)))
        })
    }

    /// Returns an iterator on all months (with their associated year) even if they don't have a balance
    fn iter_all_months(&self) -> impl Iterator<Item = (i32, MonthNum)> + '_ {
        self.balances().iter().flat_map(|(&year, balances)| {
            balances.iter().enumerate().filter_map(move |(month, _)| {
                let month = (month + 1).try_into().ok()?;
                Some((year, month))
            })
        })
    }

    fn get_balance(&self, year: i32, month: MonthNum) -> Option<i64> {
        self.balances().get(&year)?.get(&month).copied()?
    }

    fn get_balance_for_year(&self, year: i32) -> Option<BalancePerMonth> {
        self.balances().get(&year).cloned()
    }

    fn get_first_month(&self) -> Option<(i32, MonthNum)> {
        for (year, month_balances) in self.balances() {
            if let Some((month, _)) = month_balances.iter().next() {
                return Some((*year, *month));
            }
        }
        None
    }

    fn get_first_month_with_balance(&self) -> Option<(i32, MonthNum)> {
        for (year, month_balances) in self.balances() {
            for (month, &balance) in month_balances {
                if balance.is_some() {
                    return Some((*year, *month));
                }
            }
        }
        None
    }

    fn get_last_month(&self) -> Option<(i32, MonthNum)> {
        for (year, month_balances) in self.balances().iter().rev() {
            if let Some((month, _)) = month_balances.iter().next_back() {
                return Some((*year, *month));
            }
        }
        None
    }

    fn get_last_month_with_balance(&self) -> Option<(i32, MonthNum)> {
        for (year, month_balances) in self.balances().iter().rev() {
            for (month, &balance) in month_balances.iter().rev() {
                if balance.is_some() {
                    return Some((*year, *month));
                }
            }
        }
        None
    }

    fn has_year(&self, year: i32) -> bool {
        self.balances().contains_key(&year)
    }

    fn month_has_balance(&self, year: i32, month: MonthNum) -> bool {
        self.balances()
            .get(&year)
            .and_then(|month_balances| month_balances.get(&month))
            .map_or(false, |&balance| balance.is_some())
    }

    fn get_first_year(&self) -> Option<i32> {
        self.balances().keys().next().copied()
    }

    fn get_first_year_balance(&self) -> Option<BalancePerMonth> {
        self.balances().values().next().cloned()
    }

    fn get_last_year(&self) -> Option<i32> {
        self.balances().keys().next_back().copied()
    }

    fn get_last_year_balance(&self) -> Option<BalancePerMonth> {
        self.balances().values().next_back().cloned()
    }

    fn is_empty(&self) -> bool {
        self.balances().is_empty()
    }

    fn is_year_empty(&self, year: i32) -> bool {
        self.balances()
            .get(&year)
            .map_or(true, |month_balances| month_balances.is_empty())
    }

    fn clear_balances(&mut self, year: i32) {
        self.balances_mut().remove(&year);
    }

    fn clear_all_balances(&mut self) {
        self.balances_mut().clear();
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourcesToRefresh {
    pub ids: Vec<Uuid>,
}

pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
