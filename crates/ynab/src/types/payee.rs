use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Payees/getPayeeById
pub struct Payee {
    pub id: Uuid,
    pub name: String,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayeesDelta {
    pub payees: Vec<Payee>,
    pub server_knowledge: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize)]
///See https://api.youneedabudget.com/v1#/Payee_Locations/getPayeeLocationById
pub struct PayeeLocation {
    pub id: Uuid,
    pub payee_id: Uuid,
    pub latitude: String,
    pub longitude: String,
    pub deleted: bool,
}
