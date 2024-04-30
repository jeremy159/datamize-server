use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FinancialResourceType {
    Asset(AssetType),
    Liability(LiabilityType),
}

impl Default for FinancialResourceType {
    fn default() -> Self {
        Self::Asset(AssetType::default())
    }
}

impl std::fmt::Display for FinancialResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FinancialResourceType::Asset(t) => write!(f, "asset_{}", t),
            FinancialResourceType::Liability(t) => write!(f, "liability_{}", t),
        }
    }
}

impl FromStr for FinancialResourceType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asset_cash" => Ok(Self::Asset(AssetType::Cash)),
            "asset_investment" => Ok(Self::Asset(AssetType::Investment)),
            "asset_longTerm" => Ok(Self::Asset(AssetType::LongTerm)),
            "liability_cash" => Ok(Self::Liability(LiabilityType::Cash)),
            "liability_longTerm" => Ok(Self::Liability(LiabilityType::LongTerm)),
            _ => Err(format!("Failed to parse {:?} to FinancialResourceType", s)),
        }
    }
}

impl FinancialResourceType {
    pub fn category(&self) -> ResourceCategory {
        match self {
            FinancialResourceType::Asset(_) => ResourceCategory::Asset,
            FinancialResourceType::Liability(_) => ResourceCategory::Liability,
        }
    }

    pub fn is_asset(&self) -> bool {
        matches!(self, FinancialResourceType::Asset(_))
    }

    pub fn is_liability(&self) -> bool {
        matches!(self, FinancialResourceType::Liability(_))
    }

    pub fn asset_type(&self) -> Option<AssetType> {
        match self {
            FinancialResourceType::Asset(t) => Some(t.clone()),
            FinancialResourceType::Liability(_) => None,
        }
    }

    pub fn is_asset_type(&self, asset_type: AssetType) -> bool {
        match self {
            FinancialResourceType::Asset(t) => *t == asset_type,
            FinancialResourceType::Liability(_) => false,
        }
    }

    pub fn liability_type(&self) -> Option<LiabilityType> {
        match self {
            FinancialResourceType::Liability(t) => Some(t.clone()),
            FinancialResourceType::Asset(_) => None,
        }
    }

    pub fn is_liability_type(&self, liability_type: LiabilityType) -> bool {
        match self {
            FinancialResourceType::Liability(t) => *t == liability_type,
            FinancialResourceType::Asset(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub enum AssetType {
    /// Refers to current owned cash, like bank accounts.
    #[default]
    Cash,
    /// Refers to invested money, usually in the market.
    Investment,
    /// Refers to money related to house, vehicules or other long term holdings.
    LongTerm,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AssetType::Cash => write!(f, "cash"),
            AssetType::Investment => write!(f, "investment"),
            AssetType::LongTerm => write!(f, "longTerm"),
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub enum LiabilityType {
    /// Refers to current due cash, like credit cards.
    #[default]
    Cash,
    /// Refers to money related to house, vehicules or other long term holdings.
    LongTerm,
}

impl std::fmt::Display for LiabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LiabilityType::Cash => write!(f, "cash"),
            LiabilityType::LongTerm => write!(f, "longTerm"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceCategoryOption {
    pub value: ResourceCategory,
    pub selected: bool,
}

pub fn get_res_cat_options(resource_type: &FinancialResourceType) -> [ResourceCategoryOption; 2] {
    [
        ResourceCategoryOption {
            value: ResourceCategory::Asset,
            selected: resource_type.is_asset(),
        },
        ResourceCategoryOption {
            value: ResourceCategory::Liability,
            selected: resource_type.is_liability(),
        },
    ]
}

#[derive(Debug, Clone)]
pub struct ResourceTypeOption {
    pub value: String,
    pub selected: bool,
}

pub fn get_res_type_options(
    category: ResourceCategory,
    resource_type: &Option<FinancialResourceType>,
) -> Vec<ResourceTypeOption> {
    let mut common = vec![
        ResourceTypeOption {
            value: AssetType::Cash.to_string(),
            selected: resource_type.as_ref().map_or(false, |rt| {
                rt.is_asset_type(AssetType::Cash) || rt.is_liability_type(LiabilityType::Cash)
            }),
        },
        ResourceTypeOption {
            value: AssetType::LongTerm.to_string(),
            selected: resource_type.as_ref().map_or(false, |rt| {
                rt.is_asset_type(AssetType::LongTerm)
                    || rt.is_liability_type(LiabilityType::LongTerm)
            }),
        },
    ];
    if category == ResourceCategory::Asset {
        common.push(ResourceTypeOption {
            value: AssetType::Investment.to_string(),
            selected: resource_type
                .as_ref()
                .map_or(false, |rt| rt.is_asset_type(AssetType::Investment)),
        });
    }

    common
}
