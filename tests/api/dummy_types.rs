use std::str::FromStr;

use fake::{Dummy, Fake};
use rand::distributions::OpenClosed01;
use rand::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum DummyNetTotalType {
    Asset,
    Portfolio,
}

impl FromStr for DummyNetTotalType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset" => Ok(Self::Asset),
            "portfolio" => Ok(Self::Portfolio),
            _ => Err(format!("Failed to parse {:?} to NetTotalType", s)),
        }
    }
}

impl std::fmt::Display for DummyNetTotalType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DummyNetTotalType::Asset => write!(f, "asset"),
            DummyNetTotalType::Portfolio => write!(f, "portfolio"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummyNetTotal {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub net_type: DummyNetTotalType,
    // Usually i64, but using i32 to avoid overflow when we are adding stuff with randomly generated numbers.
    pub total: i32,
    #[dummy(faker = "rand::thread_rng().sample::<f32, _>(OpenClosed01)")]
    pub percent_var: f32,
    pub balance_var: i64,
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummySavingRatesPerPerson {
    pub id: Uuid,
    pub name: String,
    pub savings: i64,
    pub employer_contribution: i64,
    pub employee_contribution: i64,
    pub mortgage_capital: i64,
    pub incomes: i64,
    #[dummy(faker = "rand::thread_rng().sample::<f32, _>(OpenClosed01)")]
    pub rate: f32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum DummyResourceCategory {
    Asset,
    Liability,
}

impl std::fmt::Display for DummyResourceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DummyResourceCategory::Asset => write!(f, "asset"),
            DummyResourceCategory::Liability => write!(f, "liability"),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum DummyResourceType {
    Cash,
    Investment,
    LongTerm,
}

impl std::fmt::Display for DummyResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DummyResourceType::Cash => write!(f, "cash"),
            DummyResourceType::Investment => write!(f, "investment"),
            DummyResourceType::LongTerm => write!(f, "longTerm"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummyFinancialResource {
    pub id: Uuid,
    pub name: String,
    pub category: DummyResourceCategory,
    #[serde(rename = "type")]
    pub resource_type: DummyResourceType,
    // i32 to not overvlow when testing additions
    pub balance: i32,
    pub editable: bool,
}

#[derive(Debug, Clone, Serialize, Dummy)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum DummyAccountType {
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

#[derive(Debug, Clone, Serialize, Dummy)]
pub struct DummyAccount {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: DummyAccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    // i32 to not overvlow when testing additions
    pub balance: i32,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub transfer_payee_id: Uuid,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    pub deleted: bool,
}
