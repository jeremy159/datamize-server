use serde::{Deserialize, Serialize};
use serde_repr::*;
use uuid::Uuid;

use super::{FinancialResourceMonthly, NetTotal, ResourceCategory, ResourceType};

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(
    Serialize_repr,
    Deserialize_repr,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    Debug,
    Clone,
    Copy,
    Hash,
    sqlx::Type,
)]
#[repr(i16)]
pub enum MonthNum {
    January = 1,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl TryFrom<i16> for MonthNum {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::January),
            2 => Ok(Self::February),
            3 => Ok(Self::March),
            4 => Ok(Self::April),
            5 => Ok(Self::May),
            6 => Ok(Self::June),
            7 => Ok(Self::July),
            8 => Ok(Self::August),
            9 => Ok(Self::September),
            10 => Ok(Self::October),
            11 => Ok(Self::November),
            12 => Ok(Self::December),
            _ => Err(format!("Failed to convert {:?} to MonthNum", value)),
        }
    }
}

impl TryFrom<u32> for MonthNum {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::January),
            2 => Ok(Self::February),
            3 => Ok(Self::March),
            4 => Ok(Self::April),
            5 => Ok(Self::May),
            6 => Ok(Self::June),
            7 => Ok(Self::July),
            8 => Ok(Self::August),
            9 => Ok(Self::September),
            10 => Ok(Self::October),
            11 => Ok(Self::November),
            12 => Ok(Self::December),
            _ => Err(format!("Failed to convert {:?} to MonthNum", value)),
        }
    }
}

impl MonthNum {
    /// The next month
    pub fn succ(&self) -> MonthNum {
        match *self {
            MonthNum::January => MonthNum::February,
            MonthNum::February => MonthNum::March,
            MonthNum::March => MonthNum::April,
            MonthNum::April => MonthNum::May,
            MonthNum::May => MonthNum::June,
            MonthNum::June => MonthNum::July,
            MonthNum::July => MonthNum::August,
            MonthNum::August => MonthNum::September,
            MonthNum::September => MonthNum::October,
            MonthNum::October => MonthNum::November,
            MonthNum::November => MonthNum::December,
            MonthNum::December => MonthNum::January,
        }
    }

    pub fn pred(&self) -> MonthNum {
        match *self {
            MonthNum::January => MonthNum::December,
            MonthNum::February => MonthNum::January,
            MonthNum::March => MonthNum::February,
            MonthNum::April => MonthNum::March,
            MonthNum::May => MonthNum::April,
            MonthNum::June => MonthNum::May,
            MonthNum::July => MonthNum::June,
            MonthNum::August => MonthNum::July,
            MonthNum::September => MonthNum::August,
            MonthNum::October => MonthNum::September,
            MonthNum::November => MonthNum::October,
            MonthNum::December => MonthNum::November,
        }
    }

    pub fn name(&self) -> &'static str {
        match *self {
            MonthNum::January => "January",
            MonthNum::February => "February",
            MonthNum::March => "March",
            MonthNum::April => "April",
            MonthNum::May => "May",
            MonthNum::June => "June",
            MonthNum::July => "July",
            MonthNum::August => "August",
            MonthNum::September => "September",
            MonthNum::October => "October",
            MonthNum::November => "November",
            MonthNum::December => "December",
        }
    }
}

/// A balance sheet of the month.
#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Month {
    pub id: Uuid,
    /// The year in which the month is
    pub year: i32,
    /// The month in the year, starting with January at 1.
    pub month: MonthNum,
    /// Net Assets summary section. Includes the variation with the previous month.
    pub net_assets: NetTotal,
    /// Net Portfolio summary section. Includes the variation with the previous month.
    pub net_portfolio: NetTotal,
    /// The financial resources associated with this month only. Each resource contains a single balance for the current month
    /// even if it has occurences in other months
    pub resources: Vec<FinancialResourceMonthly>,
}

impl Month {
    pub fn new(month: MonthNum, year: i32) -> Self {
        Month {
            id: Uuid::new_v4(),
            month,
            year,
            net_assets: NetTotal::new_asset(),
            net_portfolio: NetTotal::new_portfolio(),
            resources: vec![],
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

    pub fn compute_net_totals(&mut self) {
        self.net_assets.total = self
            .resources
            .iter()
            .map(|r| match r.base.category {
                ResourceCategory::Asset => r.balance,
                ResourceCategory::Liability => -r.balance,
            })
            .sum();

        self.net_portfolio.total = self
            .resources
            .iter()
            .filter(|r| {
                r.base.category == ResourceCategory::Asset
                    && r.base.r_type != ResourceType::LongTerm
            })
            .map(|r| r.balance)
            .sum();
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize)]
pub struct SaveMonth {
    pub month: MonthNum,
}
