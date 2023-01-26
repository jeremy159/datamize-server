use rrule::{Frequency, RRule, Unvalidated};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Accounts/getAccountById
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    pub balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub transfer_payee_id: String,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    pub deleted: bool,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Accounts/getAccountById
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub account_type: AccountType,
    pub on_budget: bool,
    pub closed: bool,
    pub note: Option<String>,
    pub balance: i64,
    pub cleared_balance: i64,
    pub uncleared_balance: i64,
    pub transfer_payee_id: String,
    pub direct_import_linked: Option<bool>,
    pub direct_import_in_error: Option<bool>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsDelta {
    pub accounts: Vec<Account>,
    pub server_knowledge: i64,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum AccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    LineOfCredit,
    OtherAsset,
    OtherLiability,
    Mortgage,
    AutoLoan,
    StudentLoan,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
#[sqlx(type_name = "account_type")]
#[sqlx(rename_all = "camelCase")]
pub enum AccountType {
    Checking,
    Savings,
    Cash,
    CreditCard,
    LineOfCredit,
    OtherAsset,
    OtherLiability,
    Mortgage,
    AutoLoan,
    StudentLoan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Accounts/createAccount
pub struct SaveAccount {
    pub name: String,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub balance: i64,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ClearedType {
    Cleared,
    Uncleared,
    Reconciled,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
#[sqlx(type_name = "cleared")]
#[sqlx(rename_all = "camelCase")]
pub enum ClearedType {
    Cleared,
    Uncleared,
    Reconciled,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionSummary {
    pub id: String,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: String,
    pub payee_id: Option<String>,
    pub category_id: Option<String>,
    pub transfer_account_id: Option<String>,
    pub transfer_transaction_id: Option<String>,
    pub matched_transaction_id: Option<String>,
    pub import_id: Option<String>,
    pub deleted: bool,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionSummary {
    pub id: String,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: String,
    pub payee_id: Option<String>,
    pub category_id: Option<String>,
    pub transfer_account_id: Option<String>,
    pub transfer_transaction_id: Option<String>,
    pub matched_transaction_id: Option<String>,
    pub import_id: Option<String>,
    pub deleted: bool,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseTransactionDetail {
    pub id: String,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: String,
    pub payee_id: Option<String>,
    pub category_id: Option<String>,
    pub transfer_account_id: Option<String>,
    pub transfer_transaction_id: Option<String>,
    pub matched_transaction_id: Option<String>,
    pub import_id: Option<String>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: String,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BaseTransactionDetail {
    pub id: String,
    pub date: String,
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: String,
    pub payee_id: Option<String>,
    pub category_id: Option<String>,
    pub transfer_account_id: Option<String>,
    pub transfer_transaction_id: Option<String>,
    pub matched_transaction_id: Option<String>,
    pub import_id: Option<String>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: String,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct HybridTransaction {
    #[serde(flatten)]
    pub base: BaseTransactionDetail,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct HybridTransaction {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub base: BaseTransactionDetail,
}

impl AsRef<BaseTransactionDetail> for HybridTransaction {
    fn as_ref(&self) -> &BaseTransactionDetail {
        &self.base
    }
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionDetail {
    #[serde(flatten)]
    pub base: BaseTransactionDetail,
    pub subtransactions: Vec<SubTransaction>,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionDetail {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub base: BaseTransactionDetail,
    pub subtransactions: Vec<SubTransaction>,
}

impl AsRef<BaseTransactionDetail> for TransactionDetail {
    fn as_ref(&self) -> &BaseTransactionDetail {
        &self.base
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransationsDetailDelta<T: AsRef<BaseTransactionDetail>> {
    pub transactions: Vec<T>,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/createTransaction
pub struct SaveTransaction {
    pub account_id: String,
    pub date: String,
    pub amount: i64,
    pub payee_id: Option<String>,
    pub payee_name: Option<String>,
    pub category_id: Option<String>,
    pub memo: Option<String>,
    pub cleared: Option<ClearedType>,
    pub approved: Option<bool>,
    pub flag_color: Option<String>,
    pub import_id: Option<String>,
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/updateTransactions
pub struct UpdateTransaction {
    pub id: String,
    pub account_id: String,
    pub date: String,
    pub amount: i64,
    pub payee_id: Option<String>,
    pub payee_name: Option<String>,
    pub category_id: Option<String>,
    pub memo: Option<String>,
    pub cleared: Option<ClearedType>,
    pub approved: Option<bool>,
    pub flag_color: Option<String>,
    pub import_id: Option<String>,
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTransaction {
    pub id: Uuid,
    pub scheduled_transaction_id: Option<Uuid>,
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Transactions/createTransaction
pub struct SaveSubTransaction {
    pub amount: i64,
    pub payee_id: Option<String>,
    pub payee_name: Option<String>,
    pub category_id: Option<String>,
    pub memo: Option<String>,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecurFrequency {
    Never,
    Daily,
    Weekly,
    EveryOtherWeek,
    TwiceAMonth,
    Every4Weeks,
    Monthly,
    EveryOtherMonth,
    Every3Months,
    Every4Months,
    TwiceAYear,
    Yearly,
    EveryOtherYear,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "frequency")]
#[sqlx(rename_all = "camelCase")]
pub enum RecurFrequency {
    Never,
    Daily,
    Weekly,
    EveryOtherWeek,
    TwiceAMonth,
    Every4Weeks,
    Monthly,
    EveryOtherMonth,
    Every3Months,
    Every4Months,
    TwiceAYear,
    Yearly,
    EveryOtherYear,
}

impl RecurFrequency {
    /// Create a `RecurrenceRule` from the given YNAB frequency if applicable.
    ///
    /// Note: `RecurFrequency::Never` will produce `None`.
    pub fn as_rfc5545_rule(&self) -> Option<RRule<Unvalidated>> {
        Some(match self {
            RecurFrequency::Never => return None,
            RecurFrequency::Daily => RRule::new(Frequency::Daily),
            RecurFrequency::Weekly => RRule::new(Frequency::Weekly),
            RecurFrequency::Monthly => RRule::new(Frequency::Monthly),
            RecurFrequency::Yearly => RRule::new(Frequency::Yearly),

            RecurFrequency::EveryOtherWeek => RRule::new(Frequency::Weekly).interval(2),

            RecurFrequency::TwiceAMonth => {
                RRule::new(Frequency::Monthly).by_month_day(vec![15, -1])
            }

            RecurFrequency::Every4Weeks => RRule::new(Frequency::Weekly).interval(4),

            RecurFrequency::EveryOtherMonth => RRule::new(Frequency::Monthly).interval(2),
            RecurFrequency::Every3Months => RRule::new(Frequency::Monthly).interval(3),
            RecurFrequency::Every4Months => RRule::new(Frequency::Monthly).interval(4),

            RecurFrequency::TwiceAYear => RRule::new(Frequency::Monthly)
                .by_month(&[chrono::Month::June, chrono::Month::December]),

            RecurFrequency::EveryOtherYear => RRule::new(Frequency::Yearly).interval(2),
        })
    }
}

impl fmt::Display for RecurFrequency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RecurFrequency::Never => write!(f, "never"),
            RecurFrequency::Daily => write!(f, "daily"),
            RecurFrequency::Weekly => write!(f, "weekly"),
            RecurFrequency::Monthly => write!(f, "monthly"),
            RecurFrequency::Yearly => write!(f, "yearly"),
            RecurFrequency::EveryOtherWeek => write!(f, "everyOtherWeek"),
            RecurFrequency::TwiceAMonth => write!(f, "twiceAMonth"),
            RecurFrequency::Every4Weeks => write!(f, "every4Weeks"),
            RecurFrequency::EveryOtherMonth => write!(f, "everyOtherMonth"),
            RecurFrequency::Every3Months => write!(f, "every3Months"),
            RecurFrequency::Every4Months => write!(f, "every4Months"),
            RecurFrequency::TwiceAYear => write!(f, "twiceAYear"),
            RecurFrequency::EveryOtherYear => write!(f, "everyOtherYear"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseRecurFrequencyError;

impl FromStr for RecurFrequency {
    type Err = ParseRecurFrequencyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(Self::Never),
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "yearly" => Ok(Self::Yearly),
            "everyOtherWeek" => Ok(Self::EveryOtherWeek),
            "twiceAMonth" => Ok(Self::TwiceAMonth),
            "every4Weeks" => Ok(Self::Every4Weeks),
            "everyOtherMonth" => Ok(Self::EveryOtherMonth),
            "every3Months" => Ok(Self::Every3Months),
            "every4Months" => Ok(Self::Every4Months),
            "twiceAYear" => Ok(Self::TwiceAYear),
            "everyOtherYear" => Ok(Self::EveryOtherYear),
            _ => Err(ParseRecurFrequencyError),
        }
    }
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Scheduled_Transactions/getScheduledTransactionById
pub struct ScheduledTransactionSummary {
    pub id: String,
    pub date_first: chrono::NaiveDate,
    pub date_next: chrono::NaiveDate,
    pub frequency: Option<RecurFrequency>,
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub account_id: String,
    pub payee_id: Option<String>,
    pub category_id: Option<String>,
    pub transfer_account_id: Option<String>,
    pub deleted: bool,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Scheduled_Transactions/getScheduledTransactionById
pub struct ScheduledTransactionSummary {
    pub id: Uuid,
    pub date_first: chrono::NaiveDate,
    pub date_next: chrono::NaiveDate,
    pub frequency: Option<RecurFrequency>,
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
}

impl From<ScheduledTransactionDetail> for ScheduledTransactionSummary {
    fn from(st: ScheduledTransactionDetail) -> Self {
        Self {
            id: st.id,
            date_first: st.date_first,
            date_next: st.date_next,
            frequency: st.frequency,
            amount: st.amount,
            memo: st.memo,
            flag_color: st.flag_color,
            account_id: st.account_id,
            payee_id: st.payee_id,
            category_id: st.category_id,
            transfer_account_id: st.transfer_account_id,
            deleted: st.deleted,
        }
    }
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Scheduled_Transactions/getScheduledTransactionById
pub struct ScheduledTransactionDetail {
    pub id: Uuid,
    pub date_first: chrono::NaiveDate,
    pub date_next: chrono::NaiveDate,
    pub frequency: Option<RecurFrequency>,
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<SubTransaction>,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Scheduled_Transactions/getScheduledTransactionById
pub struct ScheduledTransactionDetail {
    pub id: Uuid,
    pub date_first: chrono::NaiveDate,
    pub date_next: chrono::NaiveDate,
    pub frequency: Option<RecurFrequency>,
    pub amount: i64,
    pub memo: Option<String>,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<SubTransaction>,
}

impl ScheduledTransactionDetail {
    pub fn from_subtransaction(self, sub_t: &SubTransaction) -> Self {
        Self {
            subtransactions: vec![],
            id: sub_t.id,
            amount: sub_t.amount,
            memo: sub_t.memo.clone(),
            payee_id: sub_t.payee_id,
            category_id: sub_t.category_id,
            transfer_account_id: sub_t.transfer_account_id,
            deleted: sub_t.deleted,
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransactionsDetailDelta {
    pub scheduled_transactions: Vec<ScheduledTransactionDetail>,
    pub server_knowledge: i64,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthSummary {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthSummary {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthSummaryDelta {
    pub months: Vec<MonthSummary>,
    pub server_knowledge: i64,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthDetail {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
    pub categories: Vec<Category>,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Months/getBudgetMonth
pub struct MonthDetail {
    pub month: String,
    pub note: Option<String>,
    pub income: i64,
    pub budgeted: i64,
    pub activity: i64,
    pub to_be_budgeted: i64,
    pub age_of_money: Option<i64>,
    pub deleted: bool,
    pub categories: Vec<Category>,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Categories/getCategoryById
pub struct Category {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    pub budgeted: i64,
    pub activity: i64,
    pub balance: i64,
    pub goal_type: Option<GoalType>,
    pub goal_creation_month: Option<chrono::NaiveDate>,
    pub goal_target: i64,
    pub goal_target_month: Option<chrono::NaiveDate>,
    pub goal_percentage_complete: Option<i64>,
    pub goal_months_to_budget: Option<i64>,
    pub goal_under_funded: Option<i64>,
    pub goal_overall_funded: Option<i64>,
    pub goal_overall_left: Option<i64>,
    pub deleted: bool,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Categories/getCategoryById
pub struct Category {
    pub id: Uuid,
    pub category_group_id: Uuid,
    pub name: String,
    pub hidden: bool,
    pub original_category_group_id: Option<Uuid>,
    pub note: Option<String>,
    pub budgeted: i64,
    pub activity: i64,
    pub balance: i64,
    pub goal_type: Option<GoalType>,
    pub goal_creation_month: Option<chrono::NaiveDate>,
    pub goal_target: i64,
    pub goal_target_month: Option<chrono::NaiveDate>,
    pub goal_percentage_complete: Option<i64>,
    pub goal_months_to_budget: Option<i64>,
    pub goal_under_funded: Option<i64>,
    pub goal_overall_funded: Option<i64>,
    pub goal_overall_left: Option<i64>,
    pub deleted: bool,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalType {
    #[serde(rename = "TB")]
    TargetBalance,
    #[serde(rename = "TBD")]
    TargetBalanceByDate,
    #[serde(rename = "MF")]
    MonthlyFunding,
    #[serde(rename = "NEED")]
    PlanYourSpending,
    #[serde(rename = "DEBT")]
    Debt,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "goal_type")]
pub enum GoalType {
    #[serde(rename = "TB")]
    #[sqlx(rename = "TB")]
    TargetBalance,
    #[serde(rename = "TBD")]
    #[sqlx(rename = "TBD")]
    TargetBalanceByDate,
    #[serde(rename = "MF")]
    #[sqlx(rename = "MF")]
    MonthlyFunding,
    #[serde(rename = "NEED")]
    #[sqlx(rename = "NEED")]
    PlanYourSpending,
    #[serde(rename = "DEBT")]
    #[sqlx(rename = "DEBT")]
    Debt,
}

impl fmt::Display for GoalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GoalType::TargetBalance => write!(f, "TB"),
            GoalType::TargetBalanceByDate => write!(f, "TBD"),
            GoalType::MonthlyFunding => write!(f, "MF"),
            GoalType::PlanYourSpending => write!(f, "NEED"),
            GoalType::Debt => write!(f, "DEBT"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseGoalTypeError;

impl FromStr for GoalType {
    type Err = ParseGoalTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TB" => Ok(Self::TargetBalance),
            "TBD" => Ok(Self::TargetBalanceByDate),
            "MF" => Ok(Self::MonthlyFunding),
            "NEED" => Ok(Self::PlanYourSpending),
            "DEBT" => Ok(Self::Debt),
            _ => Err(ParseGoalTypeError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Categories/updateMonthCategory
pub struct SaveMonthCategory {
    pub budgeted: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroup {
    pub id: String,
    pub name: String,
    pub hidden: Option<bool>,
    pub deleted: bool,
    pub transfer_account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroupWithCategories {
    pub id: String,
    pub name: String,
    pub hidden: Option<bool>,
    pub deleted: bool,
    pub transfer_account_id: Option<String>,
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGroupWithCategoriesDelta {
    pub category_groups: Vec<CategoryGroupWithCategories>,
    pub server_knowledge: i64,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Payees/getPayeeById
pub struct Payee {
    pub id: String,
    pub name: String,
    pub transfer_account_id: Option<String>,
    pub deleted: bool,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
/// See https://api.youneedabudget.com/v1#/Payees/getPayeeById
pub struct Payee {
    pub id: String,
    pub name: String,
    pub transfer_account_id: Option<String>,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayeesDelta {
    pub payees: Vec<Payee>,
    pub server_knowledge: i64,
}

#[cfg(not(feature = "sqlx-postgres"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
///See https://api.youneedabudget.com/v1#/Payee_Locations/getPayeeLocationById
pub struct PayeeLocation {
    pub id: String,
    pub payee_id: String,
    pub latitude: String,
    pub longitude: String,
    pub deleted: bool,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
///See https://api.youneedabudget.com/v1#/Payee_Locations/getPayeeLocationById
pub struct PayeeLocation {
    pub id: String,
    pub payee_id: String,
    pub latitude: String,
    pub longitude: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyFormat {
    pub iso_code: String,
    pub example_format: String,
    pub decimal_digits: i64,
    pub decimal_separator: String,
    pub symbol_first: bool,
    pub group_separator: String,
    pub currency_symbol: String,
    pub display_symbol: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateFormat {
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseBudgetSumary {
    pub id: String,
    pub name: String,
    pub last_modified_on: String,
    pub first_month: String,
    pub last_month: String,
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct BudgetSummary {
    #[serde(flatten)]
    pub base: BaseBudgetSumary,
}

impl AsRef<BaseBudgetSumary> for BudgetSummary {
    fn as_ref(&self) -> &BaseBudgetSumary {
        &self.base
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct BudgetSummaryWithAccounts {
    #[serde(flatten)]
    pub base: BaseBudgetSumary,
    pub accounts: Vec<Account>,
}

impl AsRef<BaseBudgetSumary> for BudgetSummaryWithAccounts {
    fn as_ref(&self) -> &BaseBudgetSumary {
        &self.base
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct BudgetDetail {
    pub id: String,
    pub name: String,
    pub last_modified_on: String,
    pub first_month: String,
    pub last_month: String,
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
    pub accounts: Vec<Account>,
    pub payees: Vec<Payee>,
    pub payee_locations: Vec<PayeeLocation>,
    pub category_groups: Vec<CategoryGroup>,
    pub categories: Vec<Category>,
    pub months: Vec<MonthDetail>,
    pub transactions: Vec<TransactionSummary>,
    pub subtransactions: Vec<SubTransaction>,
    pub scheduled_transactions: Vec<ScheduledTransactionSummary>,
    pub scheduled_subtransactions: Vec<SubTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDetailDelta {
    pub budget: BudgetDetail,
    pub server_knowledge: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetSettingsById
pub struct BudgetSettings {
    pub date_format: DateFormat,
    pub currency_format: CurrencyFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// See https://api.youneedabudget.com/v1#/User/getUser
pub struct User {
    pub id: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum TransactionType {
    Unapproved,
    Uncategorized,
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TransactionType::Unapproved => write!(f, "unapproved"),
            TransactionType::Uncategorized => write!(f, "uncategorized"),
        }
    }
}

/// Used for transactions request, when we need to get transactions
/// from a specific sub-path.
/// This means for example using `TransactionsParentPath::Accounts("1234")` will
/// result in the api path `"/budgets/11111/accounts/1234/transactions"`.
pub enum TransactionsParentPath<T: AsRef<str>> {
    Accounts(T),
    Categories(T),
    Payees(T),
}

#[derive(Serialize, Default)]
pub struct TransactionsRequestQuery {
    pub last_knowledge_of_server: Option<i64>,
    pub since_date: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
}

impl TransactionsRequestQuery {
    pub fn with_last_knowledge(mut self, last_knowledge_of_server: Option<i64>) -> Self {
        self.last_knowledge_of_server = last_knowledge_of_server;
        self
    }

    pub fn with_date(mut self, since_date: &str) -> Self {
        self.since_date = Some(since_date.to_string());
        self
    }

    pub fn with_transaction_type(mut self, transaction_type: TransactionType) -> Self {
        self.transaction_type = Some(transaction_type);
        self
    }
}
