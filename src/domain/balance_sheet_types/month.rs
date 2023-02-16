use serde::{Deserialize, Serialize};
use serde_repr::*;
use uuid::Uuid;

use super::{FinancialResource, NetTotal, NetTotalType, ResourceCategory, ResourceType};

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
#[derive(Debug, Serialize, Deserialize, Clone)]
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
            net_totals: vec![NetTotal::new_asset(), NetTotal::new_portfolio()],
            resources: vec![
                FinancialResource::new_bank_accounts(),
                FinancialResource::new_tfsa_jeremy(),
                FinancialResource::new_tfsa_sandryne(),
                FinancialResource::new_rrsp_jeremy(),
                FinancialResource::new_rpp_sandryne(),
                FinancialResource::new_resp(),
                FinancialResource::new_house_value(),
                FinancialResource::new_car_value(),
                FinancialResource::new_credit_cards(),
                FinancialResource::new_mortgage(),
                FinancialResource::new_cars_loan(),
            ],
        }
    }

    pub fn update_net_totals_with_previous(&mut self, prev_net_totals: &[NetTotal]) {
        for nt in &mut self.net_totals {
            if let Some(pnt) = prev_net_totals
                .iter()
                .find(|&pnt| pnt.net_type == nt.net_type)
            {
                nt.balance_var = nt.total - pnt.total;
                nt.percent_var = nt.balance_var as f32 / pnt.total as f32;
            }
        }
    }

    pub fn compute_net_totals(&mut self) {
        for nt in &mut self.net_totals {
            if nt.net_type == NetTotalType::Asset {
                nt.total = self.resources.iter().map(|r| r.balance).sum();
            } else if nt.net_type == NetTotalType::Portfolio {
                nt.total = self
                    .resources
                    .iter()
                    .filter(|r| {
                        r.category == ResourceCategory::Asset
                            && r.resource_type != ResourceType::LongTerm
                    })
                    .map(|r| r.balance)
                    .sum()
            }
        }
    }

    pub fn update_financial_resources(&mut self, resources: Vec<FinancialResource>) {
        self.resources = resources;
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SaveMonth {
    pub month: MonthNum,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateMonth {
    pub resources: Vec<FinancialResource>,
}
