use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use datamize_domain::{
    async_trait, db::ynab::DynYnabCategoryRepo, CategoryIdToNameMap, DatamizeScheduledTransaction,
    ScheduledTransactionsDistribution, Uuid,
};
use futures::{stream::FuturesUnordered, StreamExt};
use ynab::CategoryRequests;

use crate::{error::DatamizeResult, services::budget_providers::DynScheduledTransactionService};

#[async_trait]
pub trait TemplateTransactionServiceExt: Send + Sync {
    async fn get_template_transactions(&self) -> DatamizeResult<ScheduledTransactionsDistribution>;
}

pub type DynTemplateTransactionService = Arc<dyn TemplateTransactionServiceExt>;

#[derive(Clone)]
pub struct TemplateTransactionService {
    pub scheduled_transaction_service: DynScheduledTransactionService,
    pub ynab_category_repo: DynYnabCategoryRepo,
    pub ynab_client: Arc<dyn CategoryRequests + Sync + Send>,
}

impl TemplateTransactionService {
    pub fn new_arced(
        scheduled_transaction_service: DynScheduledTransactionService,
        ynab_category_repo: DynYnabCategoryRepo,
        ynab_client: Arc<dyn CategoryRequests + Sync + Send>,
    ) -> Arc<Self> {
        Arc::new(TemplateTransactionService {
            scheduled_transaction_service,
            ynab_category_repo,
            ynab_client,
        })
    }

    fn get_subtransactions_category_ids(
        scheduled_transactions: &[DatamizeScheduledTransaction],
    ) -> Vec<Uuid> {
        scheduled_transactions
            .iter()
            .flat_map(|st| &st.subtransactions)
            .filter_map(|sub_st| sub_st.category_id)
            .collect()
    }
}

#[async_trait]
impl TemplateTransactionServiceExt for TemplateTransactionService {
    #[tracing::instrument(skip(self))]
    async fn get_template_transactions(&self) -> DatamizeResult<ScheduledTransactionsDistribution> {
        let saved_scheduled_transactions = self
            .scheduled_transaction_service
            .get_latest_scheduled_transactions()
            .await?;

        let category_ids = TemplateTransactionService::get_subtransactions_category_ids(
            &saved_scheduled_transactions,
        );

        let mut category_id_to_name_map: CategoryIdToNameMap = HashMap::new();

        let categories_stream = category_ids
            .iter()
            .map(|cat_id| self.ynab_category_repo.get(*cat_id))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        for (index, category) in categories_stream.into_iter().enumerate() {
            let category = match category.context(format!(
                "failed to find category {} in database",
                category_ids[index]
            )) {
                Ok(cat) => cat,
                Err(_) => self
                    .ynab_client
                    .get_category_by_id(&category_ids[index].to_string())
                    .await
                    .context(format!(
                        "failed to get category {} in ynab",
                        category_ids[index]
                    ))?,
            };
            category_id_to_name_map.insert(category.id, category.name);
        }

        let data = ScheduledTransactionsDistribution::builder(saved_scheduled_transactions)
            .with_category_map(category_id_to_name_map)
            .build();

        Ok(data)
    }
}
