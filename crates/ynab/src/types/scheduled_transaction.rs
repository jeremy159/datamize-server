use rrule::{Frequency, RRule, Unvalidated};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(type_name = "frequency"))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(rename_all = "camelCase"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecurFrequency {
    Never,
    Daily,
    Weekly,
    EveryOtherWeek,
    TwiceAMonth,
    Every4Weeks,
    Monthly,
    EveryOtherMonth,
    Every3Months,
    Every4Months,
    TwiceAYear,
    Yearly,
    EveryOtherYear,
}

impl RecurFrequency {
    /// Create a `RecurrenceRule` from the given YNAB frequency if applicable.
    ///
    /// Note: `RecurFrequency::Never` will produce `None`.
    pub fn as_rfc5545_rule(&self) -> Option<RRule<Unvalidated>> {
        Some(match self {
            RecurFrequency::Never => return None,
            RecurFrequency::Daily => RRule::new(Frequency::Daily),
            RecurFrequency::Weekly => RRule::new(Frequency::Weekly),
            RecurFrequency::Monthly => RRule::new(Frequency::Monthly),
            RecurFrequency::Yearly => RRule::new(Frequency::Yearly),

            RecurFrequency::EveryOtherWeek => RRule::new(Frequency::Weekly).interval(2),

            RecurFrequency::TwiceAMonth => {
                RRule::new(Frequency::Monthly).by_month_day(vec![15, -1])
            }

            RecurFrequency::Every4Weeks => RRule::new(Frequency::Weekly).interval(4),

            RecurFrequency::EveryOtherMonth => RRule::new(Frequency::Monthly).interval(2),
            RecurFrequency::Every3Months => RRule::new(Frequency::Monthly).interval(3),
            RecurFrequency::Every4Months => RRule::new(Frequency::Monthly).interval(4),

            RecurFrequency::TwiceAYear => RRule::new(Frequency::Yearly)
                .by_month(&[chrono::Month::June, chrono::Month::December]),

            RecurFrequency::EveryOtherYear => RRule::new(Frequency::Yearly).interval(2),
        })
    }
}

impl fmt::Display for RecurFrequency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RecurFrequency::Never => write!(f, "never"),
            RecurFrequency::Daily => write!(f, "daily"),
            RecurFrequency::Weekly => write!(f, "weekly"),
            RecurFrequency::Monthly => write!(f, "monthly"),
            RecurFrequency::Yearly => write!(f, "yearly"),
            RecurFrequency::EveryOtherWeek => write!(f, "everyOtherWeek"),
            RecurFrequency::TwiceAMonth => write!(f, "twiceAMonth"),
            RecurFrequency::Every4Weeks => write!(f, "every4Weeks"),
            RecurFrequency::EveryOtherMonth => write!(f, "everyOtherMonth"),
            RecurFrequency::Every3Months => write!(f, "every3Months"),
            RecurFrequency::Every4Months => write!(f, "every4Months"),
            RecurFrequency::TwiceAYear => write!(f, "twiceAYear"),
            RecurFrequency::EveryOtherYear => write!(f, "everyOtherYear"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseRecurFrequencyError;

impl FromStr for RecurFrequency {
    type Err = ParseRecurFrequencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(Self::Never),
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "yearly" => Ok(Self::Yearly),
            "everyOtherWeek" => Ok(Self::EveryOtherWeek),
            "twiceAMonth" => Ok(Self::TwiceAMonth),
            "every4Weeks" => Ok(Self::Every4Weeks),
            "everyOtherMonth" => Ok(Self::EveryOtherMonth),
            "every3Months" => Ok(Self::Every3Months),
            "every4Months" => Ok(Self::Every4Months),
            "twiceAYear" => Ok(Self::TwiceAYear),
            "everyOtherYear" => Ok(Self::EveryOtherYear),
            _ => Err(ParseRecurFrequencyError),
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Scheduled_Transactions/getScheduledTransactionById
pub struct ScheduledTransactionSummary {
    pub id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date_first: chrono::NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date_next: chrono::NaiveDate,
    pub frequency: RecurFrequency,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Scheduled_Transactions/getScheduledTransactionById
pub struct ScheduledTransactionDetail {
    pub id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date_first: chrono::NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date_next: chrono::NaiveDate,
    pub frequency: RecurFrequency,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<ScheduledSubTransaction>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduledSubTransaction {
    pub id: Uuid,
    pub scheduled_transaction_id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransactionsDetailDelta {
    pub scheduled_transactions: Vec<ScheduledTransactionDetail>,
    pub server_knowledge: i64,
}
