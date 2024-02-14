use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{BaseFinancialResource, FinancialResourceType};

/// A resource represented with a month of a particular year. It has a single balance field.
#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct FinancialResourceMonthly {
    #[serde(flatten)]
    pub base: BaseFinancialResource,
    /// The balance of the resource in the month.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub balance: i64,
}

impl FinancialResourceMonthly {
    pub fn new(
        id: Uuid,
        name: String,
        resource_type: FinancialResourceType,
        ynab_account_ids: Option<Vec<Uuid>>,
        external_account_ids: Option<Vec<Uuid>>,
    ) -> Self {
        Self {
            base: BaseFinancialResource::new(
                name,
                resource_type,
                ynab_account_ids,
                external_account_ids,
            )
            .with_id(id),
            ..Default::default()
        }
    }

    pub fn with_balance(self, balance: i64) -> Self {
        Self { balance, ..self }
    }
}
