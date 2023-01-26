use std::collections::{BTreeMap, HashMap};

use crate::config::types::{ExpanseType, ExternalExpanse, SubExpanseType};
use serde::{Deserialize, Serialize, Serializer};

use uuid::Uuid;
use ynab::types::{
    RecurFrequency, ScheduledTransactionDetail, ScheduledTransactionSummary, SubTransaction,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expanse {
    pub id: Option<Uuid>,
    pub is_external: bool,
    pub name: String,
    /// The type the expanse relates to.
    #[serde(rename = "type")]
    pub expanse_type: ExpanseType,
    /// The sub_type the expanse relates to. This can be useful for example to group only housing expanses together.
    #[serde(rename = "sub_type")]
    pub sub_expanse_type: SubExpanseType,
    /// Will either be the goal_under_funded, the goal_target for the month or the amount of the linked scheduled transaction coming in the month.
    pub projected_amount: i64,
    /// At the begining of the month, this amount will be the same as projected_amount,
    /// but it will get updated during the month when some expanses occur in the category.
    pub current_amount: i64,
    /// The proportion the projected amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    pub projected_proportion: f64,
    /// The proportion the current amount represents relative to the total monthly income (salaries + health insurance + work-related RRSP)
    pub current_proportion: f64,
}

impl Expanse {
    pub fn new(
        id: Uuid,
        name: String,
        expanse_type: ExpanseType,
        sub_expanse_type: SubExpanseType,
        projected_amount: i64,
        current_amount: i64,
    ) -> Self {
        Self {
            id: Some(id),
            is_external: false,
            name,
            expanse_type,
            sub_expanse_type,
            projected_amount,
            current_amount,
            projected_proportion: 0.0,
            current_proportion: 0.0,
        }
    }
}

impl From<ExternalExpanse> for Expanse {
    fn from(value: ExternalExpanse) -> Self {
        Self {
            id: None,
            is_external: true,
            name: value.name,
            projected_amount: value.projected_amount,
            projected_proportion: 0.0,
            current_amount: value.projected_amount,
            current_proportion: 0.0,
            expanse_type: value.expanse_type,
            sub_expanse_type: value.sub_expanse_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetadata {
    /// Salary related incomes
    pub monthly_income: i64,
    /// Total income, before substracting health insurance and work-related retirement savings
    pub total_monthly_income: i64,
    /// The tartet each expanse type should follow. For example, all fixed expanses shouldn't go over 60% of total income.
    pub proportion_target_per_expanse_type: HashMap<ExpanseType, f64>,
}

impl Default for GlobalMetadata {
    fn default() -> Self {
        let tuples = [
            (ExpanseType::Fixed, 0.6_f64),
            (ExpanseType::Variable, 0.1_f64),
            (ExpanseType::ShortTermSaving, 0.1_f64),
            (ExpanseType::LongTermSaving, 0.1_f64),
            (ExpanseType::RetirementSaving, 0.1_f64),
        ];
        let proportion_target_per_expanse_type = tuples.into_iter().collect();

        Self {
            monthly_income: 0,
            total_monthly_income: 0,
            proportion_target_per_expanse_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetDetails {
    pub global: GlobalMetadata,
    pub expanses: Vec<Expanse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommonExpanseEstimationPerPerson {
    pub name: String,
    pub salary: i64,
    pub salary_per_month: i64,
    pub proportion: f64,
    pub common_expanses: i64,
    pub individual_expanses: i64,
    pub left_over: i64,
}

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

pub type ScheduledTransactionsDistributionMap =
    HashMap<String, Vec<ExtendedScheduledTransactionDetail>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTransactionsDistribution {
    #[serde(serialize_with = "ordered_map")]
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
