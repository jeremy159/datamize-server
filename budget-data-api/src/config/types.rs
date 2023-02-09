use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExpenseType {
    Fixed,
    Variable,
    ShortTermSaving,
    LongTermSaving,
    RetirementSaving,
    Undefined,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum SubExpenseType {
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
pub struct ExternalExpense {
    pub id: Option<String>,
    pub name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    pub expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    #[serde(rename = "sub_type")]
    pub sub_expense_type: SubExpenseType,
    /// Will either be the goal_under_funded or the amount of the linked scheduled transaction coming in the month
    pub projected_amount: i64,
}

#[derive(Serialize, Deserialize)]
pub struct FixedExpenses {
    pub housing_ids: Vec<Uuid>,
    pub transport_ids: Vec<Uuid>,
    pub other_ids: Vec<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct VariableExpenses {
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
pub struct BugdetCalculationDataConfig {
    pub fixed_expenses: FixedExpenses,
    pub variable_expenses: VariableExpenses,
    pub short_term_savings: ShortTermSavings,
    pub long_term_savings: LongTermSavings,
    pub retirement_savings: RetirementSavings,
    pub external_expenses: Vec<ExternalExpense>,
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
