use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::NetTotals;

use super::{Month, NetTotal};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Year {
    pub id: Uuid,
    /// The year of the date, in format 2015.
    pub year: i32,
    /// The last time a refreshed occured.
    pub refreshed_at: DateTime<Utc>,
    /// The final total net assets & portfolio of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_totals: NetTotals,
    /// All the months of the year.
    pub months: Vec<Month>,
}

impl Year {
    pub fn net_assets(&self) -> &NetTotal {
        &self.net_totals.assets
    }

    pub fn net_portfolio(&self) -> &NetTotal {
        &self.net_totals.portfolio
    }

    pub fn new(year: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            year,
            refreshed_at: Utc::now(),
            net_totals: NetTotals::default(),
            months: vec![],
        }
    }

    pub fn compute_variation(&mut self, prev_year: &Year) {
        self.net_totals.compute_variation(&prev_year.net_totals);
    }

    pub fn get_last_month(&self) -> Option<Month> {
        self.months.last().cloned()
    }

    pub fn update_net_totals(&mut self) {
        if let Some(ref month) = self.get_last_month() {
            if self.needs_update(month) {
                self.net_totals.assets.total = month.net_assets().total;
                self.net_totals.portfolio.total = month.net_portfolio().total;
            }
        }
    }

    pub(crate) fn needs_update(&self, month: &Month) -> bool {
        self.net_assets().total != month.net_assets().total
            || self.net_portfolio().total != month.net_portfolio().total
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveYear {
    /// The year of the date, in format 2015.
    pub year: i32,
}

#[cfg(any(feature = "testutils", test))]
impl fake::Dummy<fake::Faker> for Year {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        use fake::{Fake, Faker};
        let id = Fake::fake_with_rng(&Faker, rng);
        let year = Fake::fake_with_rng(&(1000..3000), rng);
        let refreshed_at = Fake::fake_with_rng(&Faker, rng);
        let net_totals = Fake::fake_with_rng(&Faker, rng);
        let months = Fake::fake_with_rng(&Faker, rng);

        Self {
            id,
            year,
            refreshed_at,
            net_totals,
            months,
        }
    }
}
