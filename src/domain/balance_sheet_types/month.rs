use serde::{Deserialize, Serialize};
use serde_repr::*;
use uuid::Uuid;

use super::{FinancialResource, NetTotal};

#[derive(
    Serialize_repr, Deserialize_repr, PartialEq, Eq, Ord, PartialOrd, Debug, Clone, Copy, sqlx::Type,
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
#[derive(Debug, Serialize)]
pub struct Month {
    pub id: Uuid,
    /// The month in the year, starting with January at 1.
    pub month: MonthNum,
    /// Net Assets or Net Portfolio summary section. Includes the variation with the previous month.
    pub net_totals: Vec<NetTotal>,
    /// All of the Assets and Liabilities of the current month are regrouped here.
    pub resources: Vec<FinancialResource>,
}

impl Month {
    pub fn new(month: MonthNum) -> Self {
        Month {
            id: Uuid::new_v4(),
            month,
            net_totals: vec![],
            resources: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveMonth {
    pub month: MonthNum,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMonth {
    pub resources: Vec<FinancialResource>,
}
