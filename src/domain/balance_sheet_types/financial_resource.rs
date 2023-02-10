use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
// #[sqlx(rename_all = "camelCase")]
pub enum ResourceCategory {
    /// Things you own. These can be cash or something you can convert into cash such as property, vehicles, equipment and inventory.
    Asset,
    /// Any financial expense or amount owed.
    Liability,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
// #[sqlx(type_name = "type")]
// #[sqlx(rename_all = "camelCase")]
pub enum ResourceType {
    /// Refers to current cash, owned or due, like bank accounts or credit cards.
    Cash,
    /// Refers to invested money, usually in the market.
    Investement,
    /// Refers to money related to house, vehicules or other long term holdings.
    LongTerm,
}

/// A resource with economic value. It represents either an asset or a liability
/// and adds more data to it.
#[derive(Debug, Serialize)]
pub struct FinancialResource {
    /// ID of the resource to be used when an update is needed.
    pub id: Uuid,
    /// The name of the resource.
    pub name: String,
    /// The category separates the resource in 2 groups: Assets vs Liabilities.
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

impl FinancialResource {
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
