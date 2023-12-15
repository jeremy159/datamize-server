use std::{fmt, str::FromStr};

use secrecy::{CloneableSecret, DebugSecret, Secret, Zeroize};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Default)]
#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
pub struct EncryptedPassword(Vec<u8>);

impl EncryptedPassword {
    pub fn new(password: Vec<u8>) -> Self {
        Self(password)
    }
}

impl AsRef<[u8]> for EncryptedPassword {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Zeroize for EncryptedPassword {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl CloneableSecret for EncryptedPassword {}
impl DebugSecret for EncryptedPassword {}

pub type SecretPassword = Secret<EncryptedPassword>;

#[derive(Clone, Debug)]
pub struct WebScrapingAccount {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub balance: i64,
    pub username: String,
    pub encrypted_password: SecretPassword,
    pub deleted: bool,
}

impl Default for WebScrapingAccount {
    fn default() -> Self {
        Self {
            id: Uuid::default(),
            name: String::default(),
            account_type: AccountType::default(),
            balance: i64::default(),
            username: String::default(),
            encrypted_password: SecretPassword::new(EncryptedPassword::default()),
            deleted: bool::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
#[sqlx(type_name = "account_type")]
#[sqlx(rename_all = "camelCase")]
pub enum AccountType {
    Tfsa, // = CELI
    Rrsp, // = REER
    Rpp,  // = RPA
    Resp, // REEE
    #[default]
    OtherAsset,
    OtherLiability,
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountType::Tfsa => write!(f, "tfsa"),
            AccountType::Rrsp => write!(f, "rrsp"),
            AccountType::Rpp => write!(f, "rpp"),
            AccountType::Resp => write!(f, "resp"),
            AccountType::OtherAsset => write!(f, "other-asset"),
            AccountType::OtherLiability => write!(f, "other-liability"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAccountTypeError;

impl FromStr for AccountType {
    type Err = ParseAccountTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tfsa" => Ok(AccountType::Tfsa),
            "rrsp" => Ok(AccountType::Rrsp),
            "rpp" => Ok(AccountType::Rpp),
            "resp" => Ok(AccountType::Resp),
            "other-asset" => Ok(AccountType::OtherAsset),
            "other-liability" => Ok(AccountType::OtherLiability),
            _ => Err(ParseAccountTypeError),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
pub struct ExternalAccount {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub balance: i64,
    pub deleted: bool,
}

impl From<WebScrapingAccount> for ExternalAccount {
    fn from(value: WebScrapingAccount) -> Self {
        ExternalAccount {
            id: value.id,
            name: value.name,
            account_type: value.account_type,
            balance: value.balance,
            deleted: value.deleted,
        }
    }
}
