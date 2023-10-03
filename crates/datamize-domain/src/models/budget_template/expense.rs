use std::{fmt, str::FromStr};

use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::{Category, GoalType};

use super::{
    Budgeter, BudgeterExt, ComputedSalary, DatamizeScheduledTransaction, ExpenseCategorization,
    ExternalExpense,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Expense<S: ExpenseState> {
    id: Uuid,
    name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    #[serde(rename = "sub_type")]
    sub_expense_type: SubExpenseType,
    /// To indicate if the expense comes from manually entered expenses, i.e. external to YNAB's data.
    is_external: bool,
    /// The individual associated with the expense. This is used to let know this expense is associated with a person in particular.
    individual_associated: Option<String>,
    #[serde(skip)]
    category: Option<Category>,
    #[serde(skip)]
    scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    #[serde(flatten)]
    extra: S,
}

impl<S: ExpenseState> Expense<S> {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn expense_type(&self) -> &ExpenseType {
        &self.expense_type
    }

    pub fn sub_expense_type(&self) -> &SubExpenseType {
        &self.sub_expense_type
    }

    pub fn is_external(&self) -> bool {
        self.is_external
    }

    pub fn individual_associated(&self) -> Option<&String> {
        self.individual_associated.as_ref()
    }

    pub fn category(&self) -> Option<&Category> {
        self.category.as_ref()
    }

    pub fn scheduled_transactions(&self) -> &[DatamizeScheduledTransaction] {
        &self.scheduled_transactions
    }

    pub fn set_categorization(mut self, expenses_categorization: &[ExpenseCategorization]) -> Self {
        match &self.category {
            Some(category) => {
                match expenses_categorization
                    .iter()
                    .find(|c| c.id == category.category_group_id)
                {
                    Some(categorization) => {
                        self.expense_type = categorization.expense_type.clone();
                        self.sub_expense_type = categorization.sub_expense_type.clone();
                        self
                    }
                    None => self,
                }
            }
            None => self,
        }
    }

    pub fn set_individual_association(mut self, budgeters: &[Budgeter<ComputedSalary>]) -> Self {
        self.individual_associated = budgeters
            .iter()
            .find(|b| self.name.contains(b.name()))
            .map(|b| b.name().to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Uncomputed;

impl Expense<Uncomputed> {
    pub fn with_scheduled_transactions<T: Into<DatamizeScheduledTransaction>>(
        mut self,
        scheduled_transactions: Vec<T>,
    ) -> Self {
        self.scheduled_transactions = scheduled_transactions.into_iter().map(Into::into).collect();
        self
    }

    pub fn compute_amounts(mut self) -> Expense<PartiallyComputed> {
        Expense {
            extra: PartiallyComputed {
                projected_amount: self.compute_projected_amount(),
                current_amount: self.compute_current_amount(),
            },
            id: self.id,
            name: self.name,
            expense_type: self.expense_type,
            sub_expense_type: self.sub_expense_type,
            is_external: self.is_external,
            category: self.category,
            individual_associated: self.individual_associated,
            scheduled_transactions: self.scheduled_transactions,
        }
    }

    fn compute_projected_amount(&mut self) -> i64 {
        if let Some(category) = &self.category {
            let projected_amount = match category.goal_type {
                Some(GoalType::Debt) => 0, // Debt type goal should not be considered in the amount as they arlready have a scheduled transaction of the same amount
                Some(GoalType::PlanYourSpending) => {
                    match (category.goal_cadence, category.goal_cadence_frequency) {
                        (Some(1), Some(freq)) => category.goal_target / freq as i64,
                        (Some(1), None) => category.goal_target,
                        (Some(cadence), _) => category.goal_target / (cadence - 1) as i64,
                        (_, _) => 0,
                    }
                }
                Some(_) => category.goal_target,
                None => 0,
            };

            return projected_amount
                + self
                    .scheduled_transactions
                    .iter()
                    .map(|v| -v.amount)
                    .sum::<i64>();
        }

        0
    }

    fn compute_current_amount(&mut self) -> i64 {
        if let Some(category) = &self.category {
            let current_amount_budgeted = match category.goal_type {
                Some(_) => match category.goal_under_funded {
                    // If goal was fully funded, simply return what was budgeted
                    Some(0) => category.budgeted,
                    // If goal was partially funded, add the budgeted amount + what is left to reach goal
                    Some(i) => i + category.budgeted,
                    None => 0,
                },
                None => category.budgeted,
            };

            let mut current_amount = current_amount_budgeted;
            let scheduled_transactions_total = self
                .scheduled_transactions
                .iter()
                .map(|v| -v.amount)
                .sum::<i64>();

            if current_amount_budgeted != scheduled_transactions_total {
                let current_date = Local::now().date_naive();
                let future_transactions_amount = self
                    .scheduled_transactions
                    .iter()
                    .filter(|_| category.goal_type.is_none())
                    // Current amount should only take into account scheduled transactions from future. those in past should instead be taken from budgeted section.
                    .filter(|&st| st.date_next > current_date)
                    .map(|st| -st.amount)
                    .sum::<i64>();

                if future_transactions_amount > 0 {
                    current_amount = future_transactions_amount;
                }
            }

            return current_amount;
        }

        0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PartiallyComputed {
    /// Will either be the goal_under_funded, the goal_target for the month or the amount of the linked scheduled transaction coming in the month.
    projected_amount: i64,
    /// At the begining of the month, this amount will be the same as projected_amount,
    /// but it will get updated during the month when some expenses occur in the category.
    current_amount: i64,
}

impl Expense<PartiallyComputed> {
    pub fn projected_amount(&self) -> i64 {
        self.extra.projected_amount
    }

    pub fn current_amount(&self) -> i64 {
        self.extra.current_amount
    }

    pub fn compute_proportions(self, total_income: i64) -> Expense<Computed> {
        Expense {
            extra: Computed {
                projected_proportion: self.extra.projected_amount as f64 / total_income as f64,
                current_proportion: self.extra.current_amount as f64 / total_income as f64,
                partially_computed: PartiallyComputed {
                    projected_amount: self.extra.projected_amount,
                    current_amount: self.extra.current_amount,
                },
            },
            id: self.id,
            name: self.name,
            expense_type: self.expense_type,
            sub_expense_type: self.sub_expense_type,
            is_external: self.is_external,
            category: self.category,
            individual_associated: self.individual_associated,
            scheduled_transactions: self.scheduled_transactions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Computed {
    #[serde(flatten)]
    partially_computed: PartiallyComputed,
    /// The proportion the projected amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    projected_proportion: f64,
    /// The proportion the current amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    current_proportion: f64,
}

impl Expense<Computed> {
    pub fn projected_amount(&self) -> i64 {
        self.extra.partially_computed.projected_amount
    }

    pub fn current_amount(&self) -> i64 {
        self.extra.partially_computed.current_amount
    }

    pub fn projected_proportion(&self) -> f64 {
        self.extra.projected_proportion
    }

    pub fn current_proportion(&self) -> f64 {
        self.extra.current_proportion
    }
}

impl From<Category> for Expense<Uncomputed> {
    fn from(value: Category) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            is_external: false,
            category: Some(value),
            ..Default::default()
        }
    }
}

impl From<ExternalExpense> for Expense<PartiallyComputed> {
    fn from(value: ExternalExpense) -> Self {
        Self {
            id: value.id,
            name: value.name,
            extra: PartiallyComputed {
                projected_amount: value.projected_amount,
                current_amount: value.projected_amount,
            },
            expense_type: value.expense_type,
            sub_expense_type: value.sub_expense_type,
            is_external: true,
            ..Default::default()
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Default, sqlx::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum ExpenseType {
    Fixed,
    Variable,
    ShortTermSaving,
    LongTermSaving,
    RetirementSaving,
    #[default]
    Undefined,
}

impl fmt::Display for ExpenseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExpenseType::Fixed => write!(f, "fixed"),
            ExpenseType::Variable => write!(f, "variable"),
            ExpenseType::ShortTermSaving => write!(f, "shortTermSaving"),
            ExpenseType::LongTermSaving => write!(f, "longTermSaving"),
            ExpenseType::RetirementSaving => write!(f, "retirementSaving"),
            ExpenseType::Undefined => write!(f, "undefined"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseExpenseTypeError;

impl FromStr for ExpenseType {
    type Err = ParseExpenseTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fixed" => Ok(Self::Fixed),
            "variable" => Ok(Self::Variable),
            "shortTermSaving" => Ok(Self::ShortTermSaving),
            "longTermSaving" => Ok(Self::LongTermSaving),
            "retirementSaving" => Ok(Self::RetirementSaving),
            "undefined" => Ok(Self::Undefined),
            _ => Err(ParseExpenseTypeError),
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash, sqlx::Type,
)]
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
    #[default]
    Undefined,
}

impl fmt::Display for SubExpenseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SubExpenseType::Housing => write!(f, "housing"),
            SubExpenseType::Transport => write!(f, "transport"),
            SubExpenseType::OtherFixed => write!(f, "otherFixed"),
            SubExpenseType::Subscription => write!(f, "subscription"),
            SubExpenseType::OtherVariable => write!(f, "otherVariable"),
            SubExpenseType::ShortTermSaving => write!(f, "shortTermSaving"),
            SubExpenseType::LongTermSaving => write!(f, "longTermSaving"),
            SubExpenseType::RetirementSaving => write!(f, "retirementSaving"),
            SubExpenseType::Undefined => write!(f, "undefined"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseSubExpenseTypeError;

impl FromStr for SubExpenseType {
    type Err = ParseSubExpenseTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "housing" => Ok(Self::Housing),
            "transport" => Ok(Self::Transport),
            "otherFixed" => Ok(Self::OtherFixed),
            "subscription" => Ok(Self::Subscription),
            "otherVariable" => Ok(Self::OtherVariable),
            "shortTermSaving" => Ok(Self::ShortTermSaving),
            "longTermSaving" => Ok(Self::LongTermSaving),
            "retirementSaving" => Ok(Self::RetirementSaving),
            "undefined" => Ok(Self::Undefined),
            _ => Err(ParseSubExpenseTypeError),
        }
    }
}

pub trait ExpenseState {}
impl ExpenseState for Uncomputed {}
impl ExpenseState for PartiallyComputed {}
impl ExpenseState for Computed {}
