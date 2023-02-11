use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
// #[sqlx(type_name = "type")]
// #[sqlx(rename_all = "camelCase")]
pub enum NetTotalType {
    /// Net Assets is the total of owned assets minus the total of liabilities.
    Asset,
    /// Net Portfolio is the total of owned assets that are tangible cash. For example, bank or investments accounts
    /// are tangible cash assets but not the value of your house or car.
    Portfolio,
}

#[derive(Debug, Serialize)]
pub struct NetTotal {
    pub id: Uuid,
    /// Internal splitting beyond the category.
    #[serde(rename = "type")]
    // #[sqlx(rename = "type")]
    pub net_type: NetTotalType,
    /// The total amount of the current section.
    pub total: i64,
    /// The percentage of variation compared to the previous month's section.
    pub percent_var: f32,
    /// The money balance of variation compared to the previous month's section.
    pub balance_var: i64,
}
