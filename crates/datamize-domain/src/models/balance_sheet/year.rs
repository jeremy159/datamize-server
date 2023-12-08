use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Month, NetTotal};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Year {
    pub id: Uuid,
    /// The year of the date, in format 2015.
    pub year: i32,
    /// The last time a refreshed occured.
    pub refreshed_at: DateTime<Utc>,
    /// The final total net assets of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_assets: NetTotal,
    /// The final total portfolio of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_portfolio: NetTotal,
    /// All the months of the year.
    pub months: Vec<Month>,
}

#[cfg(any(feature = "testutils", test))]
impl fake::Dummy<fake::Faker> for Year {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        use fake::{Fake, Faker};
        let id = Fake::fake_with_rng(&Faker, rng);
        let year = Fake::fake_with_rng(&(1000..3000), rng);
        let refreshed_at = Fake::fake_with_rng(&Faker, rng);
        let net_assets = NetTotal {
            net_type: crate::NetTotalType::Asset,
            ..Faker.fake()
        };
        let net_portfolio = NetTotal {
            net_type: crate::NetTotalType::Portfolio,
            ..Faker.fake()
        };
        let months = Fake::fake_with_rng(&Faker, rng);

        Self {
            id,
            year,
            refreshed_at,
            net_assets,
            net_portfolio,
            months,
        }
    }
}

impl Year {
    pub fn new(year: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            year,
            refreshed_at: Utc::now(),
            net_assets: NetTotal::new_asset(),
            net_portfolio: NetTotal::new_portfolio(),
            months: vec![],
        }
    }

    pub fn update_net_assets_with_previous(&mut self, prev_net_assets: &NetTotal) {
        self.net_assets.balance_var = self.net_assets.total - prev_net_assets.total;
        self.net_assets.percent_var = match prev_net_assets.total {
            0 => 0.0,
            _ => self.net_assets.balance_var as f32 / prev_net_assets.total as f32,
        };
    }

    pub fn update_net_portfolio_with_previous(&mut self, prev_net_portfolio: &NetTotal) {
        self.net_portfolio.balance_var = self.net_portfolio.total - prev_net_portfolio.total;
        self.net_portfolio.percent_var = match prev_net_portfolio.total {
            0 => 0.0,
            _ => self.net_portfolio.balance_var as f32 / prev_net_portfolio.total as f32,
        };
    }

    pub fn get_last_month(&self) -> Option<Month> {
        self.months.last().cloned()
    }

    pub fn needs_net_totals_update(
        &self,
        month_net_assets: &NetTotal,
        month_net_portfolio: &NetTotal,
    ) -> bool {
        self.net_assets.total != month_net_assets.total
            || self.net_portfolio.total != month_net_portfolio.total
    }

    pub fn update_net_assets_with_last_month(&mut self, month_net_assets: &NetTotal) {
        self.net_assets.total = month_net_assets.total;
    }

    pub fn update_net_portfolio_with_last_month(&mut self, month_net_portfolio: &NetTotal) {
        self.net_portfolio.total = month_net_portfolio.total;
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveYear {
    /// The year of the date, in format 2015.
    pub year: i32,
}
