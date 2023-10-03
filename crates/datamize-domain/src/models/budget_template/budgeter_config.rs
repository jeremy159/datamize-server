use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq)]
pub struct BudgeterConfig {
    pub id: Uuid,
    pub name: String,
    pub payee_ids: Vec<Uuid>,
}

impl BudgeterConfig {
    pub fn new(name: String, payee_ids: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            payee_ids,
        }
    }
}

impl From<SaveBudgeterConfig> for BudgeterConfig {
    fn from(value: SaveBudgeterConfig) -> Self {
        Self::new(value.name, value.payee_ids)
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveBudgeterConfig {
    pub name: String,
    pub payee_ids: Vec<Uuid>,
}
