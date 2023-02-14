use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, sqlx::Type)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
#[sqlx(type_name = "category")]
#[sqlx(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, sqlx::Type)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
#[sqlx(type_name = "resource_type")]
#[sqlx(rename_all = "camelCase")]
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

/// A resource with economic value. It represents either an asset or a liability
/// and adds more data to it.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct FinancialResource {
    /// ID of the resource to be used when an update is needed.
    pub id: Uuid,
    /// The name of the resource.
    pub name: String,
    /// The category separates the resource in 2 groups: Assets vs Liabilities.
    /// Liabilities should have a negative balance.
    pub category: ResourceCategory,
    /// Internal splitting beyond the category.
    #[serde(rename = "type")]
    // #[sqlx(rename = "type")]
    pub resource_type: ResourceType,
    /// The balance of the resource in the month.
    pub balance: i64,
    /// Flag to indicate if the resource can be edited with the API.
    pub editable: bool,
}

// TODO: customize template instead of enforcing names here...
impl FinancialResource {
    pub fn new_bank_accounts() -> Self {
        FinancialResource::new_asset("Comptes Bancaires".to_string())
            .of_type(ResourceType::Cash)
            .non_editable()
    }

    pub fn new_tfsa_jeremy() -> Self {
        FinancialResource::new_asset("CELI Jeremy".to_string())
            .of_type(ResourceType::Investment)
            .non_editable()
    }

    pub fn new_tfsa_sandryne() -> Self {
        FinancialResource::new_asset("CELI Sandryne".to_string())
            .of_type(ResourceType::Investment)
            .non_editable()
    }

    pub fn new_rrsp_jeremy() -> Self {
        FinancialResource::new_asset("REER Jeremy".to_string()).of_type(ResourceType::Investment)
    }

    pub fn new_rpp_sandryne() -> Self {
        FinancialResource::new_asset("RPA Sandryne".to_string()).of_type(ResourceType::Investment)
    }

    pub fn new_resp() -> Self {
        FinancialResource::new_asset("REEE".to_string()).of_type(ResourceType::Investment)
    }

    pub fn new_house_value() -> Self {
        FinancialResource::new_asset("Valeur Maison".to_string()).of_type(ResourceType::LongTerm)
    }

    pub fn new_car_value() -> Self {
        FinancialResource::new_asset("Valeur Automobile".to_string())
            .of_type(ResourceType::LongTerm)
    }

    pub fn new_credit_cards() -> Self {
        FinancialResource::new_liability("Cartes de Crédit".to_string())
            .of_type(ResourceType::Cash)
            .non_editable()
    }

    pub fn new_mortgage() -> Self {
        FinancialResource::new_liability("Prêt Hypothécaire".to_string())
            .of_type(ResourceType::LongTerm)
            .non_editable()
    }

    pub fn new_cars_loan() -> Self {
        FinancialResource::new_liability("Prêts Automobile".to_string())
            .of_type(ResourceType::LongTerm)
            .non_editable()
    }

    pub fn new_asset(name: String) -> Self {
        FinancialResource {
            id: Uuid::new_v4(),
            name,
            category: ResourceCategory::Asset,
            resource_type: ResourceType::Cash,
            balance: 0,
            editable: true,
        }
    }

    pub fn new_liability(name: String) -> Self {
        FinancialResource {
            id: Uuid::new_v4(),
            name,
            category: ResourceCategory::Liability,
            resource_type: ResourceType::Cash,
            balance: 0,
            editable: true,
        }
    }

    pub fn of_type(mut self, resource_type: ResourceType) -> Self {
        self.resource_type = resource_type;
        self
    }

    pub fn non_editable(mut self) -> Self {
        self.editable = false;
        self
    }

    pub fn with_balance(mut self, new_balance: i64) -> Self {
        self.balance = new_balance;
        self
    }

    pub fn add_to_balance(&mut self, balance: i64) {
        self.balance += balance;
    }

    pub fn override_balance(&mut self, balance: i64) {
        self.balance = balance;
    }
}
