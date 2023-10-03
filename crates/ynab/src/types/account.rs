use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Accounts/getAccountById
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    pub balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub transfer_payee_id: Uuid,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Accounts/getAccountById
pub struct Account {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    pub balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub transfer_payee_id: Uuid,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsDelta {
    pub accounts: Vec<Account>,
    pub server_knowledge: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg(not(feature = "sqlx-postgres"))]
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
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
#[sqlx(type_name = "account_type")]
#[sqlx(rename_all = "camelCase")]
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
    pub balance: i64,
}
