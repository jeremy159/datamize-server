use chrono::Local;
use serde::{Deserialize, Serialize};
use ynab::types::{Category, GoalType, ScheduledTransactionDetail};

use crate::config::CategoryGroup;

use super::{Budgeter, ComputedSalary};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Expense<S: ExpenseState> {
    name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    #[serde(rename = "sub_type")]
    sub_expense_type: SubExpenseType,
    /// The individual associated with the expense. This is used to let know this expense is associated with a person in particular.
    individual_associated: Option<String>,
    category: Option<Category>,
    #[serde(skip)]
    scheduled_transactions: Vec<ScheduledTransactionDetail>,
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

    pub fn individual_associated(&self) -> Option<&String> {
        self.individual_associated.as_ref()
    }

    pub fn category(&self) -> Option<&Category> {
        self.category.as_ref()
    }

    pub fn scheduled_transactions(&self) -> &[ScheduledTransactionDetail] {
        &self.scheduled_transactions
    }

    pub fn set_categorization(mut self, category_groups: &[CategoryGroup]) -> Self {
        if let Some(category) = &self.category {
            for cat_group in category_groups {
                if cat_group.ids.contains(&category.category_group_id) {
                    self.expense_type = cat_group.expense_type.clone();
                    self.sub_expense_type = cat_group.sub_expense_type.clone();
                    return self;
                }
            }
        }

        self
    }

    pub fn set_individual_association(mut self, budgeters: &[Budgeter<ComputedSalary>]) -> Self {
        self.individual_associated = budgeters
            .iter()
            .find(|b| self.name.contains(b.name()))
            .map(|b| b.name().clone());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Uncomputed;

impl Expense<Uncomputed> {
    pub fn with_scheduled_transactions(
        mut self,
        scheduled_transactions: Vec<ScheduledTransactionDetail>,
    ) -> Self {
        self.scheduled_transactions = scheduled_transactions;
        self
    }

    pub fn compute_amounts(mut self) -> Expense<PartiallyComputed> {
        Expense {
            extra: PartiallyComputed {
                projected_amount: self.compute_projected_amount(),
                current_amount: self.compute_current_amount(),
            },
            name: self.name,
            expense_type: self.expense_type,
            sub_expense_type: self.sub_expense_type,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            name: self.name,
            expense_type: self.expense_type,
            sub_expense_type: self.sub_expense_type,
            category: self.category,
            individual_associated: self.individual_associated,
            scheduled_transactions: self.scheduled_transactions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
            name: value.name.clone(),
            category: Some(value),
            ..Default::default()
        }
    }
}

impl From<ExternalExpense> for Expense<PartiallyComputed> {
    fn from(value: ExternalExpense) -> Self {
        Self {
            name: value.name,
            extra: PartiallyComputed {
                projected_amount: value.projected_amount,
                current_amount: value.projected_amount,
            },
            expense_type: value.expense_type,
            sub_expense_type: value.sub_expense_type,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalExpense {
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

pub trait ExpenseState {}
impl ExpenseState for Uncomputed {}
impl ExpenseState for PartiallyComputed {}
impl ExpenseState for Computed {}
