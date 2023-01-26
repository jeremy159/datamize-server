use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExpanseType {
    Fixed,
    Variable,
    ShortTermSaving,
    LongTermSaving,
    RetirementSaving,
    Undefined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SubExpanseType {
    Housing,
    Transport,
    OtherFixed,
    Subscription,
    OtherVariable,
    ShortTermSaving,
    LongTermSaving,
    RetirementSaving,
    Undefined,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExternalExpanse {
    pub id: Option<String>,
    pub name: String,
    /// The type the expanse relates to.
    #[serde(rename = "type")]
    pub expanse_type: ExpanseType,
    /// The sub_type the expanse relates to. This can be useful for example to group only housing expanses together.
    #[serde(rename = "sub_type")]
    pub sub_expanse_type: SubExpanseType,
    /// Will either be the goal_under_funded or the amount of the linked scheduled transaction coming in the month
    pub projected_amount: i64,
}

#[derive(Serialize, Deserialize)]
pub struct FixedExpanses {
    pub housing_ids: Vec<Uuid>,
    pub transport_ids: Vec<Uuid>,
    pub other_ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct VariableExpanses {
    pub subscription_ids: Vec<Uuid>,
    pub other_ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct ShortTermSavings {
    pub ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct LongTermSavings {
    pub ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct RetirementSavings {
    pub ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
}

impl RedisConfig {
    pub fn connection_string(&self) -> String {
        format!("redis://{}:{}/", self.host, self.port)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BugdetCalculationDataConfig {
    pub fixed_expanses: FixedExpanses,
    pub variable_expanses: VariableExpanses,
    pub short_term_savings: ShortTermSavings,
    pub long_term_savings: LongTermSavings,
    pub retirement_savings: RetirementSavings,
    pub external_expanses: Vec<ExternalExpanse>,
}

#[derive(Serialize, Deserialize)]
pub struct PersonSalaryConfig {
    pub name: String,
    pub payee_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct CeliConfig {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub login_url: String,
    pub data_url: String,
}
