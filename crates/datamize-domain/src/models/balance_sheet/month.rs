use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{FinancialResourceMonthly, NetTotal};
use crate::{MonthNum, NetTotals};

// TODO: Try to convert these model to follow the 'Fat Model' design.
// https://loco.rs/docs/the-app/models/
// https://github.com/loco-rs/loco/blob/master/examples/demo/src/models/_entities/users.rs
// https://github.com/SeaQL/sea-orm/blob/master/src/database/db_connection.rs
/// A balance sheet of the month.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Month {
    pub id: Uuid,
    /// The year in which the month is
    pub year: i32,
    /// The month in the year, starting with January at 1.
    pub month: MonthNum,
    pub net_totals: NetTotals,
    /// The financial resources associated with this month only. Each resource contains a single balance for the current month
    /// even if it has occurences in other months
    pub resources: Vec<FinancialResourceMonthly>,
}

impl Month {
    pub fn net_assets(&self) -> &NetTotal {
        &self.net_totals.assets
    }

    pub fn net_portfolio(&self) -> &NetTotal {
        &self.net_totals.portfolio
    }

    pub fn new(month: MonthNum, year: i32) -> Self {
        Month {
            id: Uuid::new_v4(),
            month,
            year,
            net_totals: NetTotals::default(),
            resources: vec![],
        }
    }

    pub fn compute_variation(&mut self, prev_month: &Month) {
        self.net_totals.compute_variation(&prev_month.net_totals);
    }

    pub fn compute_net_totals(&mut self) {
        self.net_totals
            .compute_totals_from_resources(&self.resources);
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveMonth {
    pub month: MonthNum,
}

#[cfg(any(feature = "testutils", test))]
impl fake::Dummy<fake::Faker> for Month {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &fake::Faker, rng: &mut R) -> Self {
        use fake::{Fake, Faker};
        let id = Fake::fake_with_rng(&Faker, rng);
        let year = Fake::fake_with_rng(&(1000..3000), rng);
        let month = Fake::fake_with_rng(&Faker, rng);
        let net_totals = Fake::fake_with_rng(&Faker, rng);
        let resources = Fake::fake_with_rng(&Faker, rng);

        Self {
            id,
            year,
            month,
            net_totals,
            resources,
        }
    }
}
