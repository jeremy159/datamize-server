use serde_repr::*;

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

impl From<MonthNum> for i16 {
    fn from(value: MonthNum) -> Self {
        match value {
            MonthNum::January => 1,
            MonthNum::February => 2,
            MonthNum::March => 3,
            MonthNum::April => 4,
            MonthNum::May => 5,
            MonthNum::June => 6,
            MonthNum::July => 7,
            MonthNum::August => 8,
            MonthNum::September => 9,
            MonthNum::October => 10,
            MonthNum::November => 11,
            MonthNum::December => 12,
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
