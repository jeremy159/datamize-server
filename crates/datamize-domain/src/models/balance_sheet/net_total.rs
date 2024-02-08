use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{FinancialResourceMonthly, ResourceCategory, ResourceType};

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct NetTotals {
    /// Net Assets summary section. Includes the variation with the previous month.
    pub assets: NetTotal,
    /// Net Portfolio summary section. Includes the variation with the previous month.
    pub portfolio: NetTotal,
}

impl NetTotals {
    pub fn compute_variation(&mut self, previous: &NetTotals) {
        self.assets.compute_variation(&previous.assets);
        self.portfolio.compute_variation(&previous.portfolio);
    }

    pub fn compute_totals_from_resources(&mut self, resources: &[FinancialResourceMonthly]) {
        let mut total_assets = 0;
        let mut total_portfolio = 0;
        let mut at_least_one_in_portfolio = false;

        for resource in resources {
            match resource.base.category {
                ResourceCategory::Asset => {
                    total_assets += resource.balance;
                    if resource.base.r_type != ResourceType::LongTerm {
                        at_least_one_in_portfolio = true;
                        total_portfolio += resource.balance;
                    }
                }
                ResourceCategory::Liability => total_assets -= resource.balance,
            }
        }

        if !resources.is_empty() {
            self.assets.total = total_assets;
            self.assets.last_updated = Some(Utc::now());

            if at_least_one_in_portfolio {
                self.portfolio.total = total_portfolio;
                self.portfolio.last_updated = Some(Utc::now());
            }
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NetTotal {
    pub id: Uuid,
    /// The total amount of the current section.
    #[cfg_attr(
        any(feature = "testutils", test),
        dummy(faker = "-1000000..1000000000")
    )]
    pub total: i64,
    /// The percentage of variation compared to the previous month's section.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0.0..1.0"))]
    pub percent_var: f32,
    /// The money balance of variation compared to the previous month's section.
    #[cfg_attr(
        any(feature = "testutils", test),
        dummy(faker = "-1000000..1000000000")
    )]
    pub balance_var: i64,
    pub last_updated: Option<DateTime<Utc>>,
}

impl NetTotal {
    pub fn compute_variation(&mut self, previous: &NetTotal) {
        let variation = Variation::calculate(previous.total, self.total);
        self.balance_var = variation.balance_var;
        self.percent_var = variation.percent_var;
        self.last_updated = Some(Utc::now());
    }
}

impl Default for NetTotal {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            total: 0,
            percent_var: 0.0,
            balance_var: 0,
            last_updated: None,
        }
    }
}

pub struct Variation {
    /// The money balance of variation compared to the previous month's section.
    pub balance_var: i64,
    /// The percentage of variation compared to the previous month's section.
    pub percent_var: f32,
}

impl Variation {
    pub fn calculate(previous_total: i64, current_total: i64) -> Self {
        let balance_var = current_total - previous_total;
        let percent_var = if previous_total != 0 {
            (current_total as f32 - previous_total as f32) / previous_total as f32
        } else {
            0.0
        };

        Variation {
            balance_var,
            percent_var,
        }
    }
}

#[cfg(any(feature = "testutils", test))]
pub fn net_totals_equal_without_id(a: &NetTotals, b: &NetTotals) -> bool {
    net_total_equal_without_id(&a.assets, &b.assets)
        && net_total_equal_without_id(&a.portfolio, &b.portfolio)
}

#[cfg(any(feature = "testutils", test))]
pub fn net_total_equal_without_id(a: &NetTotal, b: &NetTotal) -> bool {
    a.total == b.total
        && a.percent_var == b.percent_var
        && a.balance_var == b.balance_var
        && a.last_updated == b.last_updated
}
