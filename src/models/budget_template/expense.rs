use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::{Category, GoalType, ScheduledTransactionDetail};

use crate::config::{BugdetCalculationDataSettings, PersonSalarySettings};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Expense {
    pub id: Option<Uuid>,
    pub is_external: bool,
    pub name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    pub expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    #[serde(rename = "sub_type")]
    pub sub_expense_type: SubExpenseType,
    /// Will either be the goal_under_funded, the goal_target for the month or the amount of the linked scheduled transaction coming in the month.
    pub projected_amount: i64,
    /// At the begining of the month, this amount will be the same as projected_amount,
    /// but it will get updated during the month when some expenses occur in the category.
    pub current_amount: i64,
    /// The proportion the projected amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    pub projected_proportion: f64,
    /// The proportion the current amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    pub current_proportion: f64,
    /// The individual associated with the expense. This is used to let know this expense is associated with a person in particular.
    pub individual_associated: Option<String>,
    #[serde(skip)]
    pub category: Option<Category>,
    #[serde(skip)]
    pub scheduled_transactions: Vec<ScheduledTransactionDetail>,
}

impl Expense {
    pub fn new(
        id: Uuid,
        name: String,
        expense_type: ExpenseType,
        sub_expense_type: SubExpenseType,
        projected_amount: i64,
        current_amount: i64,
    ) -> Self {
        Self {
            id: Some(id),
            is_external: false,
            name,
            expense_type,
            sub_expense_type,
            projected_amount,
            current_amount,
            projected_proportion: 0.0,
            current_proportion: 0.0,
            category: None,
            individual_associated: None,
            scheduled_transactions: vec![],
        }
    }

    pub fn with_scheduled_transactions(
        mut self,
        scheduled_transactions: Vec<ScheduledTransactionDetail>,
    ) -> Self {
        self.scheduled_transactions = scheduled_transactions;
        self
    }

    pub fn compute_projected_amount(mut self) -> Self {
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

            self.projected_amount = projected_amount
                + self
                    .scheduled_transactions
                    .iter()
                    .map(|v| -v.amount)
                    .sum::<i64>();
        }

        self
    }

    pub fn compute_current_amount(mut self) -> Self {
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

            self.current_amount = current_amount_budgeted;
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
                    self.current_amount = future_transactions_amount;
                }
            }
        }

        self
    }

    pub fn set_categorization(
        mut self,
        budget_calculation_data_settings: &BugdetCalculationDataSettings,
    ) -> Self {
        if let Some(category) = &self.category {
            for cat_group in &budget_calculation_data_settings.category_groups {
                if cat_group.ids.contains(&category.category_group_id) {
                    self.expense_type = cat_group.expense_type.clone();
                    self.sub_expense_type = cat_group.sub_expense_type.clone();
                    return self;
                }
            }
        }

        self.expense_type = ExpenseType::Undefined;
        self.sub_expense_type = SubExpenseType::Undefined;

        self
    }

    pub fn set_individual_association(
        mut self,
        person_salary_settings: &[PersonSalarySettings],
    ) -> Self {
        self.individual_associated = person_salary_settings
            .iter()
            .find(|config| self.name.contains(&config.name))
            .map(|config| config.name.clone());
        self
    }
}

impl From<Category> for Expense {
    fn from(value: Category) -> Self {
        Self {
            id: Some(value.id),
            name: value.name.clone(),
            is_external: false,
            category: Some(value),
            ..Default::default()
        }
    }
}

impl From<ExternalExpense> for Expense {
    fn from(value: ExternalExpense) -> Self {
        Self {
            id: None,
            is_external: true,
            name: value.name,
            projected_amount: value.projected_amount,
            projected_proportion: 0.0,
            current_amount: value.projected_amount,
            current_proportion: 0.0,
            expense_type: value.expense_type,
            sub_expense_type: value.sub_expense_type,
            category: None,
            individual_associated: None,
            scheduled_transactions: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
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
