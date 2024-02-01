use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionSummary {
    pub id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date: chrono::NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<Uuid>,
    pub import_payee_name: Option<String>,
    pub import_payee_name_original: Option<String>,
    pub debt_transaction_type: Option<DebtTransactionType>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BaseTransactionDetail {
    pub id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date: chrono::NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub account_id: Uuid,
    pub payee_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub matched_transaction_id: Option<Uuid>,
    pub import_id: Option<Uuid>,
    pub import_payee_name: Option<String>,
    pub import_payee_name_original: Option<String>,
    pub debt_transaction_type: Option<DebtTransactionType>,
    pub deleted: bool,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(type_name = "cleared"))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(rename_all = "camelCase"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum ClearedType {
    Cleared,
    Uncleared,
    Reconciled,
}

impl fmt::Display for ClearedType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClearedType::Cleared => write!(f, "cleared"),
            ClearedType::Uncleared => write!(f, "uncleared"),
            ClearedType::Reconciled => write!(f, "reconciled"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseClearedTypeError;

impl FromStr for ClearedType {
    type Err = ParseClearedTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cleared" => Ok(ClearedType::Cleared),
            "uncleared" => Ok(ClearedType::Uncleared),
            "reconciled" => Ok(ClearedType::Reconciled),
            _ => Err(ParseClearedTypeError),
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(type_name = "debt_transaction"))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(rename_all = "camelCase"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum DebtTransactionType {
    Payment,
    Refund,
    Fee,
    Interest,
    Escrow,
    BalanceAdjustment,
    Credit,
    Charge,
}

impl fmt::Display for DebtTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DebtTransactionType::Payment => write!(f, "payment"),
            DebtTransactionType::Refund => write!(f, "refund"),
            DebtTransactionType::Fee => write!(f, "fee"),
            DebtTransactionType::Interest => write!(f, "interest"),
            DebtTransactionType::Escrow => write!(f, "escrow"),
            DebtTransactionType::BalanceAdjustment => write!(f, "balanceAdjustment"),
            DebtTransactionType::Credit => write!(f, "credit"),
            DebtTransactionType::Charge => write!(f, "charge"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseDebtTransactionTypeError;

impl FromStr for DebtTransactionType {
    type Err = ParseDebtTransactionTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "payment" => Ok(DebtTransactionType::Payment),
            "refund" => Ok(DebtTransactionType::Refund),
            "fee" => Ok(DebtTransactionType::Fee),
            "interest" => Ok(DebtTransactionType::Interest),
            "escrow" => Ok(DebtTransactionType::Escrow),
            "balanceAdjustment" => Ok(DebtTransactionType::BalanceAdjustment),
            "credit" => Ok(DebtTransactionType::Credit),
            "charge" => Ok(DebtTransactionType::Charge),
            _ => Err(ParseDebtTransactionTypeError),
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// See https://api.youneedabudget.com/v1#/Budgets/getBudgetById
pub struct HybridTransaction {
    #[serde(flatten)]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(flatten))]
    pub base: BaseTransactionDetail,
    #[serde(rename = "type")]
    pub transaction_type: HybridTransactionType,
    pub parent_transaction_id: Option<Uuid>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(type_name = "hybrid_transaction"))]
#[cfg_attr(feature = "sqlx-postgres", sqlx(rename_all = "camelCase"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum HybridTransactionType {
    Transaction,
    Subtransaction,
}

impl fmt::Display for HybridTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HybridTransactionType::Transaction => write!(f, "transaction"),
            HybridTransactionType::Subtransaction => write!(f, "subtransaction"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseHybridTransactionTypeError;

impl FromStr for HybridTransactionType {
    type Err = ParseHybridTransactionTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "transaction" => Ok(HybridTransactionType::Transaction),
            "subtransaction" => Ok(HybridTransactionType::Subtransaction),
            _ => Err(ParseHybridTransactionTypeError),
        }
    }
}

impl AsRef<BaseTransactionDetail> for HybridTransaction {
    fn as_ref(&self) -> &BaseTransactionDetail {
        &self.base
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridTransationsDelta {
    pub transactions: Vec<HybridTransaction>,
    pub server_knowledge: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[cfg_attr(feature = "sqlx-postgres", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// See https://api.youneedabudget.com/v1#/Transactions/getTransactionById
pub struct TransactionDetail {
    #[serde(flatten)]
    #[cfg_attr(feature = "sqlx-postgres", sqlx(flatten))]
    pub base: BaseTransactionDetail,
    pub subtransactions: Vec<SubTransaction>,
}

impl AsRef<BaseTransactionDetail> for TransactionDetail {
    fn as_ref(&self) -> &BaseTransactionDetail {
        &self.base
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransationsDetailDelta {
    pub transactions: Vec<TransactionDetail>,
    pub server_knowledge: i64,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Transactions/createTransaction
pub struct SaveTransaction {
    pub account_id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date: chrono::NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub import_id: Option<Uuid>,
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Transactions/updateTransactions
pub struct UpdateTransaction {
    pub id: Uuid,
    pub account_id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(default))]
    pub date: chrono::NaiveDate,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub memo: Option<String>,
    pub cleared: ClearedType,
    pub approved: bool,
    pub flag_color: Option<String>,
    pub import_id: Option<Uuid>,
    pub subtransactions: Option<Vec<SaveSubTransaction>>,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SubTransaction {
    pub id: Uuid,
    pub transaction_id: Uuid,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub memo: Option<String>,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub transfer_account_id: Option<Uuid>,
    pub transfer_transaction_id: Option<Uuid>,
    pub deleted: bool,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// See https://api.youneedabudget.com/v1#/Transactions/createTransaction
pub struct SaveSubTransaction {
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-100000..100000"))]
    pub amount: i64,
    pub payee_id: Option<Uuid>,
    pub payee_name: Option<String>,
    pub category_id: Option<Uuid>,
    pub memo: Option<String>,
}
