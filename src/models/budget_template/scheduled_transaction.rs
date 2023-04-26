use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize, Serializer};
use ynab::types::{
    RecurFrequency, ScheduledTransactionDetail, ScheduledTransactionSummary, SubTransaction,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedSubTransaction {
    #[serde(flatten)]
    pub subtransaction: SubTransaction,
    pub category_name: Option<String>,
    pub date_first: chrono::NaiveDate,
    pub date_next: chrono::NaiveDate,
    pub frequency: Option<RecurFrequency>,
    pub flag_color: Option<String>,
    pub account_id: uuid::Uuid,
    pub account_name: String,
    pub payee_name: Option<String>,
}

impl ExtendedSubTransaction {
    pub fn from_sub_trans_and_trans(
        sub_trans: SubTransaction,
        trans: ScheduledTransactionDetail,
    ) -> Self {
        Self {
            subtransaction: sub_trans,
            category_name: None,
            date_first: trans.date_first,
            date_next: trans.date_next,
            frequency: trans.frequency,
            flag_color: trans.flag_color,
            account_id: trans.account_id,
            account_name: trans.account_name,
            payee_name: trans.payee_name,
        }
    }

    pub fn with_category_name(self, category_name: Option<String>) -> Self {
        Self {
            category_name,
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedScheduledTransactionDetail {
    #[serde(flatten)]
    pub scheduled_transaction: ScheduledTransactionSummary,
    pub account_name: String,
    pub payee_name: Option<String>,
    pub category_name: Option<String>,
    pub subtransactions: Vec<ExtendedSubTransaction>,
}

impl From<ExtendedSubTransaction> for ExtendedScheduledTransactionDetail {
    fn from(e: ExtendedSubTransaction) -> Self {
        Self {
            scheduled_transaction: ScheduledTransactionSummary {
                id: e.subtransaction.id,
                date_first: e.date_first,
                date_next: e.date_next,
                frequency: e.frequency,
                amount: e.subtransaction.amount,
                memo: e.subtransaction.memo,
                flag_color: e.flag_color,
                account_id: e.account_id,
                payee_id: e.subtransaction.payee_id,
                category_id: e.subtransaction.category_id,
                transfer_account_id: e.subtransaction.transfer_account_id,
                deleted: e.subtransaction.deleted,
            },
            account_name: e.account_name,
            payee_name: e.payee_name,
            category_name: e.category_name,
            subtransactions: vec![],
        }
    }
}

impl From<ScheduledTransactionDetail> for ExtendedScheduledTransactionDetail {
    fn from(value: ScheduledTransactionDetail) -> Self {
        let account_name = value.account_name.clone();
        let payee_name = value.payee_name.clone();
        let category_name = value.category_name.clone();

        Self {
            scheduled_transaction: value.into(),
            account_name,
            payee_name,
            category_name,
            subtransactions: vec![],
        }
    }
}

pub type CategoryIdToNameMap = HashMap<uuid::Uuid, String>;

pub type ScheduledTransactionsDistributionMap =
    HashMap<String, Vec<ExtendedScheduledTransactionDetail>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransactionsDistribution {
    #[serde(serialize_with = "ordered_map", flatten)]
    pub map: ScheduledTransactionsDistributionMap,
}

fn ordered_map<S>(
    value: &ScheduledTransactionsDistributionMap,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
