use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize, Serializer};

use super::DatamizeScheduledTransaction;

pub type CategoryIdToNameMap = HashMap<uuid::Uuid, String>;

pub type ScheduledTransactionsDistributionMap = HashMap<String, Vec<DatamizeScheduledTransaction>>;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ScheduledTransactionsDistribution {
    #[serde(serialize_with = "ordered_map", flatten)]
    map: ScheduledTransactionsDistributionMap,
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

impl ScheduledTransactionsDistribution {
    pub fn builder(
        scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    ) -> ScheduledTransactionsDistributionBuilder {
        ScheduledTransactionsDistributionBuilder::new(scheduled_transactions)
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
        let mut scheduled_transactions = self.scheduled_transactions;
        let mut map: ScheduledTransactionsDistributionMap = HashMap::new();

        let mut repeated_sts: Vec<DatamizeScheduledTransaction> = vec![];

        for dst in scheduled_transactions.iter().filter(|dst| !dst.deleted) {
            repeated_sts.extend(dst.get_repeated_transactions());
        }

        scheduled_transactions.extend(repeated_sts);

        let scheduled_transactions: Vec<_> = scheduled_transactions
            .into_iter()
            .filter(|dst| dst.is_in_next_30_days())
            .flat_map(|dst| dst.flatten())
            .map(|dst| match dst.category_name {
                Some(_) => dst,
                None => {
                    let category_name = dst.category_id.as_ref().and_then(|id| {
                        self.category_id_to_name_map
                            .as_ref()
                            .and_then(|category_map| category_map.get(id).cloned())
                    });

                    dst.with_category_name(category_name)
                }
            })
            .collect();

        for dst in scheduled_transactions {
            let entry = map.entry(dst.date_next.to_string());
            entry.or_default().push(dst);
        }

        ScheduledTransactionsDistribution { map }
    }
}
