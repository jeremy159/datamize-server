use std::{fmt, str::FromStr};

use chrono::{DateTime, Datelike, Days, Local, Months, TimeZone};
use rrule::{Frequency, NWeekday, RRule, Tz, Weekday};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::types::{Category, GoalType};

use super::{
    Budgeter, BudgeterExt, ComputedSalary, DatamizeScheduledTransaction, ExpenseCategorization,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct WeeklyCadenceData {
    dt_start: DateTime<Tz>,
    goal_start: DateTime<Tz>,
    dt_end: DateTime<Tz>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Expense<S: ExpenseState> {
    id: Uuid,
    name: String,
    /// The type the expense relates to.
    #[serde(rename = "type")]
    expense_type: ExpenseType,
    /// The sub_type the expense relates to. This can be useful for example to group only housing expenses together.
    /// By default it will use the category group name, but it can also use the enum `SubExpenseType`
    #[serde(rename = "sub_type")]
    sub_expense_type: String,
    /// The individual associated with the expense. This is used to let know this expense is associated with a person in particular.
    individual_associated: Option<String>,
    #[serde(skip)]
    category: Category,
    #[serde(skip)]
    scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    #[serde(skip)]
    weekly_cadence_data: Option<WeeklyCadenceData>,
    #[serde(flatten)]
    extra: S,
}

impl<S: ExpenseState> Expense<S> {
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn expense_type(&self) -> &ExpenseType {
        &self.expense_type
    }

    pub fn sub_expense_type(&self) -> &str {
        &self.sub_expense_type
    }

    pub fn individual_associated(&self) -> Option<&String> {
        self.individual_associated.as_ref()
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn scheduled_transactions(&self) -> &[DatamizeScheduledTransaction] {
        &self.scheduled_transactions
    }

    pub fn build_dates(mut self) -> Self {
        if let Some(start_date) = self.category.goal_creation_month {
            // Last day previous month
            let dt_start = Local::now()
                .with_day(1)
                .and_then(|d| d.checked_sub_days(Days::new(1)))
                .and_then(|d| {
                    Tz::Local(Local)
                        .from_local_datetime(&d.naive_local())
                        .single()
                });

            let goal_start = start_date
                .and_hms_opt(0, 0, 0)
                .and_then(|d| Tz::Local(Local).from_local_datetime(&d).single());

            // Last day current month
            let dt_end = Local::now()
                .checked_add_months(Months::new(1))
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.checked_sub_days(Days::new(1)))
                .and_then(|d| {
                    Tz::Local(Local)
                        .from_local_datetime(&d.naive_local())
                        .single()
                });

            self.weekly_cadence_data = match (dt_start, goal_start, dt_end) {
                (Some(dt_start), Some(goal_start), Some(dt_end)) => Some(WeeklyCadenceData {
                    dt_start,
                    goal_start,
                    dt_end,
                }),
                (_, _, _) => None,
            };
        }
        self
    }

    pub fn set_categorization(
        mut self,
        expenses_categorization: &[ExpenseCategorization],
        use_category_groups_as_sub_type: bool,
    ) -> Self {
        match expenses_categorization
            .iter()
            .find(|c| c.id == self.category.category_group_id)
        {
            Some(categorization) => {
                self.expense_type = categorization.expense_type.clone();
                self.sub_expense_type = if use_category_groups_as_sub_type {
                    self.category.category_group_name.clone()
                } else {
                    categorization.sub_expense_type.clone().to_string()
                };
                self
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

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
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
            category: self.category,
            individual_associated: self.individual_associated,
            scheduled_transactions: self.scheduled_transactions,
            weekly_cadence_data: self.weekly_cadence_data,
        }
    }

    fn compute_projected_amount(&mut self) -> i64 {
        match self.category.goal_type {
            Some(GoalType::PlanYourSpending) => {
                match (
                    self.category.goal_cadence,
                    self.category.goal_cadence_frequency,
                    self.category.goal_target,
                ) {
                    (Some(1), Some(freq), Some(target)) if freq > 0 => target / freq as i64, // Goal repeats X months
                    (Some(1), None, Some(target)) => target, // Goal repeats monthly
                    (Some(2), Some(freq), Some(target)) if freq > 0 => self
                        .compute_monthly_target_for_weekly_goal_cadence(
                            freq as u16,
                            self.category.get_goal_day_as_weekday(),
                            target,
                        ), // Goal repeats weekly
                    (Some(cad @ 3..=13), _, Some(target)) => target / (cad - 1) as i64, // Goal repeats X months (up to yearly)
                    (Some(14), _, Some(target)) => target / 24, // Goal repeats every 2 years
                    (_, _, _) => 0,
                }
            }
            Some(_) => self.category.goal_target.unwrap_or(0),
            None => 0,
        }
    }

    fn compute_monthly_target_for_weekly_goal_cadence(
        &self,
        interval: u16,
        weekday: Option<Weekday>,
        target: i64,
    ) -> i64 {
        if let Some(dates) = &self.weekly_cadence_data {
            let mut rrule = RRule::new(Frequency::Weekly)
                .interval(interval)
                .week_start(Weekday::Sun)
                .until(dates.dt_end);

            if let Some(weekday) = weekday {
                rrule = rrule.by_weekday(vec![NWeekday::Every(weekday)])
            }

            if let Ok(rrule_set) = rrule.build(dates.dt_start) {
                let occurences = rrule_set
                    .into_iter()
                    .filter(|date| date >= &dates.goal_start)
                    .count();

                return target * occurences as i64;
            }
        }

        0
    }

    fn compute_current_amount(&mut self) -> i64 {
        let current_amount_budgeted = match self.category.goal_type {
            Some(_) => match self.category.goal_under_funded {
                // If goal was fully funded, simply return what was budgeted
                Some(0) => self.category.budgeted,
                // If goal was partially funded, add the budgeted amount + what is left to reach goal
                Some(i) => i + self.category.budgeted,
                None => 0,
            },
            None => self.category.budgeted,
        };

        let mut current_amount = if current_amount_budgeted >= 0 {
            current_amount_budgeted
        } else {
            0
        };

        if !self.scheduled_transactions.is_empty() && self.category.goal_type.is_none() {
            let scheduled_transactions_total = self
                .scheduled_transactions
                .iter()
                .map(|v| -v.amount)
                .sum::<i64>();

            if current_amount != scheduled_transactions_total {
                let current_date = Local::now().date_naive();
                let future_transactions_amount = self
                    .scheduled_transactions
                    .iter()
                    // Current amount should only take into account scheduled transactions from future. those in past should instead be taken from budgeted section.
                    .filter(|&st| st.date_next > current_date)
                    .map(|st| -st.amount)
                    .sum::<i64>();

                if future_transactions_amount > 0
                    && future_transactions_amount > self.category.balance
                {
                    current_amount = future_transactions_amount - self.category.balance;
                }
            }
        }

        current_amount
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PartiallyComputed {
    /// Will either be the goal_under_funded, the goal_target for the month or the amount of the linked scheduled transaction coming in the month.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    projected_amount: i64,
    /// At the begining of the month, this amount will be the same as projected_amount,
    /// but it will get updated during the month when some expenses occur in the category.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
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
                projected_proportion: if total_income == 0 {
                    0.0
                } else {
                    self.extra.projected_amount as f64 / total_income as f64
                },
                current_proportion: if total_income == 0 {
                    0.0
                } else {
                    self.extra.current_amount as f64 / total_income as f64
                },
                partially_computed: PartiallyComputed {
                    projected_amount: self.extra.projected_amount,
                    current_amount: self.extra.current_amount,
                },
            },
            id: self.id,
            name: self.name,
            expense_type: self.expense_type,
            sub_expense_type: self.sub_expense_type,
            category: self.category,
            individual_associated: self.individual_associated,
            scheduled_transactions: self.scheduled_transactions,
            weekly_cadence_data: self.weekly_cadence_data,
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Computed {
    #[serde(flatten)]
    partially_computed: PartiallyComputed,
    /// The proportion the projected amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0.0..1.0"))]
    projected_proportion: f64,
    /// The proportion the current amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "0.0..1.0"))]
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
            category: value,
            sub_expense_type: SubExpenseType::Undefined.to_string(),
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

#[cfg(any(feature = "testutils", test))]
impl<S: ExpenseState + fake::Dummy<fake::Faker>> fake::Dummy<fake::Faker> for Expense<S> {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &fake::Faker, rng: &mut R) -> Self {
        use fake::Fake;
        let id = config.fake_with_rng(rng);
        let name = config.fake_with_rng(rng);
        let expense_type = config.fake_with_rng(rng);
        let sub_expense_type = config.fake_with_rng(rng);
        let individual_associated = config.fake_with_rng(rng);
        let category = config.fake_with_rng(rng);
        let scheduled_transactions = config.fake_with_rng(rng);
        let extra = config.fake_with_rng(rng);
        let weekly_cadence_data = None;

        Self {
            id,
            name,
            expense_type,
            sub_expense_type,
            individual_associated,
            category,
            scheduled_transactions,
            extra,
            weekly_cadence_data,
        }
    }
}
