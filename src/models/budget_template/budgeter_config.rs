use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveBudgeterConfig {
    pub name: String,
    pub payee_ids: Vec<Uuid>,
}
