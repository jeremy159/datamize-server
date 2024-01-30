use std::{collections::HashMap, fmt, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Accounts/getAccountById
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(rename = "type"))]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub balance: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub cleared_balance: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub uncleared_balance: i64,
    pub transfer_payee_id: Uuid,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub last_reconciled_at: Option<DateTime<Utc>>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub debt_original_balance: Option<i64>,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub debt_interest_rates: HashMap<NaiveDate, i64>,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub debt_minimum_payments: HashMap<NaiveDate, i64>,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub debt_escrow_amounts: HashMap<NaiveDate, i64>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsDelta {
    pub accounts: Vec<Account>,
    pub server_knowledge: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(type_name = "account_type"))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(rename_all = "camelCase"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum AccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    LineOfCredit,
    OtherAsset,
    OtherLiability,
    Mortgage,
    AutoLoan,
    StudentLoan,
    PersonalLoan,
    MedicalDebt,
    OtherDebt,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountType::Checking => write!(f, "checking"),
            AccountType::Savings => write!(f, "savings"),
            AccountType::Cash => write!(f, "cash"),
            AccountType::CreditCard => write!(f, "creditCard"),
            AccountType::LineOfCredit => write!(f, "lineOfCredit"),
            AccountType::OtherAsset => write!(f, "otherAsset"),
            AccountType::OtherLiability => write!(f, "otherLiability"),
            AccountType::Mortgage => write!(f, "mortgage"),
            AccountType::AutoLoan => write!(f, "autoLoan"),
            AccountType::StudentLoan => write!(f, "studentLoan"),
            AccountType::PersonalLoan => write!(f, "personalLoan"),
            AccountType::MedicalDebt => write!(f, "medicalDebt"),
            AccountType::OtherDebt => write!(f, "otherDebt"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAccountTypeError;

impl FromStr for AccountType {
    type Err = ParseAccountTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "checking" => Ok(AccountType::Checking),
            "savings" => Ok(AccountType::Savings),
            "cash" => Ok(AccountType::Cash),
            "creditCard" => Ok(AccountType::CreditCard),
            "lineOfCredit" => Ok(AccountType::LineOfCredit),
            "otherAsset" => Ok(AccountType::OtherAsset),
            "otherLiability" => Ok(AccountType::OtherLiability),
            "mortgage" => Ok(AccountType::Mortgage),
            "autoLoan" => Ok(AccountType::AutoLoan),
            "studentLoan" => Ok(AccountType::StudentLoan),
            "personalLoan" => Ok(AccountType::PersonalLoan),
            "medicalDebt" => Ok(AccountType::MedicalDebt),
            "otherDebt" => Ok(AccountType::OtherDebt),
            _ => Err(ParseAccountTypeError),
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Accounts/createAccount
pub struct SaveAccount {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub balance: i64,
}
