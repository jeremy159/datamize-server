use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::MonthNum;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ResourceCategory {
    /// Things you own. These can be cash or something you can convert into cash such as property, vehicles, equipment and inventory.
    Asset,
    /// Any financial expense or amount owed.
    Liability,
}

impl std::fmt::Display for ResourceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ResourceCategory::Asset => write!(f, "asset"),
            ResourceCategory::Liability => write!(f, "liability"),
        }
    }
}

impl FromStr for ResourceCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset" => Ok(Self::Asset),
            "liability" => Ok(Self::Liability),
            _ => Err(format!("Failed to parse {:?} to ResourceCategory", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ResourceType {
    /// Refers to current cash, owned or due, like bank accounts or credit cards.
    Cash,
    /// Refers to invested money, usually in the market.
    Investment,
    /// Refers to money related to house, vehicules or other long term holdings.
    LongTerm,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ResourceType::Cash => write!(f, "cash"),
            ResourceType::Investment => write!(f, "investment"),
            ResourceType::LongTerm => write!(f, "longTerm"),
        }
    }
}

impl FromStr for ResourceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cash" => Ok(Self::Cash),
            "investment" => Ok(Self::Investment),
            "longTerm" => Ok(Self::LongTerm),
            _ => Err(format!("Failed to parse {:?} to ResourceType", s)),
        }
    }
}

/// A resource with economic value. It represents either an asset or a liability
/// and adds more data to it.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseFinancialResource {
    /// ID of the resource to be used when an update is needed.
    pub id: Uuid,
    /// The name of the resource.
    pub name: String,
    /// The category separates the resource in 2 groups: Assets vs Liabilities.
    /// Liabilities should have a negative balance.
    pub category: ResourceCategory,
    /// Internal splitting beyond the category.
    #[serde(rename = "type")]
    pub r_type: ResourceType,
    /// Flag to indicate if the resource can be edited with the API.
    pub editable: bool,
}

impl BaseFinancialResource {
    pub fn new(
        name: String,
        category: ResourceCategory,
        r_type: ResourceType,
        editable: bool,
    ) -> Self {
        BaseFinancialResource {
            id: Uuid::new_v4(),
            name,
            category,
            r_type,
            editable,
        }
    }

    pub fn new_asset(name: String, r_type: ResourceType, editable: bool) -> Self {
        BaseFinancialResource {
            id: Uuid::new_v4(),
            name,
            category: ResourceCategory::Asset,
            r_type,
            editable,
        }
    }

    pub fn new_liability(name: String, r_type: ResourceType, editable: bool) -> Self {
        BaseFinancialResource {
            id: Uuid::new_v4(),
            name,
            category: ResourceCategory::Liability,
            r_type,
            editable,
        }
    }
}

/// A resource represented within a year. It has a BTreeMap of balance per months.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinancialResourceYearly {
    #[serde(flatten)]
    pub base: BaseFinancialResource,
    /// The year in which the financial resource is
    pub year: i32,
    /// The balance of the resource in the month.
    pub balance_per_month: BTreeMap<MonthNum, i64>,
}

impl FinancialResourceYearly {
    pub fn new(
        name: String,
        category: ResourceCategory,
        r_type: ResourceType,
        editable: bool,
        year: i32,
    ) -> Self {
        FinancialResourceYearly {
            base: BaseFinancialResource::new(name, category, r_type, editable),
            year,
            balance_per_month: BTreeMap::new(),
        }
    }

    pub fn new_asset(name: String, r_type: ResourceType, editable: bool, year: i32) -> Self {
        FinancialResourceYearly {
            base: BaseFinancialResource::new_asset(name, r_type, editable),
            year,
            balance_per_month: BTreeMap::new(),
        }
    }

    pub fn new_liability(name: String, r_type: ResourceType, editable: bool, year: i32) -> Self {
        FinancialResourceYearly {
            base: BaseFinancialResource::new_liability(name, r_type, editable),
            year,
            balance_per_month: BTreeMap::new(),
        }
    }
}

/// A resource represented with a month of a particular year. It has a single balance field.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinancialResourceMonthly {
    #[serde(flatten)]
    pub base: BaseFinancialResource,
    /// The month in which the financial resource has the current balance.
    pub month: MonthNum,
    /// The year in which the financial resource is
    pub year: i32,
    /// The balance of the resource in the month.
    pub balance: i64,
}

impl FinancialResourceMonthly {
    pub fn new(
        name: String,
        category: ResourceCategory,
        r_type: ResourceType,
        editable: bool,
        month: MonthNum,
        year: i32,
    ) -> Self {
        FinancialResourceMonthly {
            base: BaseFinancialResource::new(name, category, r_type, editable),
            month,
            year,
            balance: 0,
        }
    }

    pub fn new_asset(
        name: String,
        r_type: ResourceType,
        editable: bool,
        month: MonthNum,
        year: i32,
    ) -> Self {
        FinancialResourceMonthly {
            base: BaseFinancialResource::new_asset(name, r_type, editable),
            month,
            year,
            balance: 0,
        }
    }

    pub fn new_liability(
        name: String,
        r_type: ResourceType,
        editable: bool,
        month: MonthNum,
        year: i32,
    ) -> Self {
        FinancialResourceMonthly {
            base: BaseFinancialResource::new_liability(name, r_type, editable),
            month,
            year,
            balance: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SaveResource {
    pub name: String,
    pub category: ResourceCategory,
    #[serde(rename = "type")]
    pub r_type: ResourceType,
    pub editable: bool,
    pub year: i32,
    pub balance_per_month: BTreeMap<MonthNum, i64>,
    // TODO: Maybe also add possibility to save ynab accounts linked to it.
}

impl From<SaveResource> for FinancialResourceYearly {
    fn from(value: SaveResource) -> Self {
        FinancialResourceYearly {
            base: BaseFinancialResource::new(
                value.name,
                value.category,
                value.r_type,
                value.editable,
            ),
            year: value.year,
            balance_per_month: value.balance_per_month,
        }
    }
}
