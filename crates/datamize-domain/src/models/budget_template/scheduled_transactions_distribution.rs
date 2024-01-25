use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::DatamizeScheduledTransaction;

pub type CategoryIdToNameMap = HashMap<uuid::Uuid, String>;

pub type ScheduledTransactionsDistributionMap =
    BTreeMap<NaiveDate, Vec<DatamizeScheduledTransaction>>;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ScheduledTransactionsDistribution {
    #[serde(flatten)]
    map: ScheduledTransactionsDistributionMap,
}

impl ScheduledTransactionsDistribution {
    pub fn builder(
        scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    ) -> ScheduledTransactionsDistributionBuilder {
        ScheduledTransactionsDistributionBuilder::new(scheduled_transactions)
    }

    pub fn map(&self) -> &ScheduledTransactionsDistributionMap {
        &self.map
    }
}

pub struct ScheduledTransactionsDistributionBuilder {
    scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    category_id_to_name_map: Option<CategoryIdToNameMap>,
}

impl ScheduledTransactionsDistributionBuilder {
    pub fn new(scheduled_transactions: Vec<DatamizeScheduledTransaction>) -> Self {
        Self {
            scheduled_transactions: scheduled_transactions
                .into_iter()
                .flat_map(|dst| dst.flatten())
                .collect(),
            category_id_to_name_map: None,
        }
    }

    pub fn with_category_map(mut self, category_id_to_name_map: CategoryIdToNameMap) -> Self {
        self.category_id_to_name_map = Some(category_id_to_name_map);
        self
    }

    pub fn build(self) -> ScheduledTransactionsDistribution {
        let mut scheduled_transactions = self
            .scheduled_transactions
            .into_par_iter()
            .filter(|t| !t.deleted)
            .collect::<Vec<_>>();

        for i in 0..scheduled_transactions.len() {
            let dst = &scheduled_transactions[i];

            if let Some(repeated_trans) = dst.get_repeated_transactions() {
                scheduled_transactions.extend(repeated_trans);
            }
        }

        let scheduled_transactions: Vec<_> = scheduled_transactions
            .into_par_iter()
            .filter(|dst| !dst.deleted && dst.is_in_next_30_days().unwrap_or(false))
            .flat_map(|dst| dst.flatten())
            .map(|dst| {
                let category_name = dst.category_name.clone().or_else(|| {
                    dst.category_id.as_ref().and_then(|id| {
                        self.category_id_to_name_map
                            .as_ref()
                            .and_then(|category_map| category_map.get(id))
                            .cloned()
                    })
                });

                dst.with_category_name(category_name)
            })
            .collect();

        let mut map: ScheduledTransactionsDistributionMap = BTreeMap::new();

        for dst in scheduled_transactions {
            let entry = map
                .entry(dst.date_next)
                .or_insert_with(|| Vec::with_capacity(1));
            entry.push(dst);
        }

        ScheduledTransactionsDistribution { map }
    }
}
