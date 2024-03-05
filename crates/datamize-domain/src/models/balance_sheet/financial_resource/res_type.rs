use std::str::FromStr;

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

    pub fn asset_type(&self) -> Option<AssetType> {
        match self {
            FinancialResourceType::Asset(t) => Some(t.clone()),
            FinancialResourceType::Liability(_) => None,
        }
    }

    pub fn liability_type(&self) -> Option<LiabilityType> {
        match self {
            FinancialResourceType::Liability(t) => Some(t.clone()),
            FinancialResourceType::Asset(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceCategory {
    /// Things you own. These can be cash or something you can convert into cash such as property, vehicles, equipment and inventory.
    Asset,
    /// Any financial expense or amount owed.
    Liability,
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
