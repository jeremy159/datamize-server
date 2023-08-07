use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use ynab::CategoryRequests;

use crate::{
    db::budget_providers::ynab::YnabCategoryRepo,
    error::DatamizeResult,
    models::budget_template::{
        CategoryIdToNameMap, DatamizeScheduledTransaction, ScheduledTransactionsDistribution,
    },
};

use super::ScheduledTransactionServiceExt;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TemplateTransactionServiceExt {
    async fn get_template_transactions(
        &mut self,
    ) -> DatamizeResult<ScheduledTransactionsDistribution>;
}

pub struct TemplateTransactionService<YCR: YnabCategoryRepo> {
    pub scheduled_transaction_service: Box<dyn ScheduledTransactionServiceExt + Sync + Send>,
    pub ynab_category_repo: YCR,
    pub ynab_client: Arc<dyn CategoryRequests + Sync + Send>,
}

#[async_trait]
impl<YCR> TemplateTransactionServiceExt for TemplateTransactionService<YCR>
where
    YCR: YnabCategoryRepo + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_template_transactions(
        &mut self,
    ) -> DatamizeResult<ScheduledTransactionsDistribution> {
        let saved_scheduled_transactions = self
            .scheduled_transaction_service
            .get_latest_scheduled_transactions()
            .await?;

        let category_ids = TemplateTransactionService::<YCR>::get_subtransactions_category_ids(
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

impl<YCR: YnabCategoryRepo> TemplateTransactionService<YCR> {
    fn get_subtransactions_category_ids(
        scheduled_transactions: &[DatamizeScheduledTransaction],
    ) -> Vec<uuid::Uuid> {
        scheduled_transactions
            .iter()
            .flat_map(|st| &st.subtransactions)
            .filter_map(|sub_st| sub_st.category_id)
            .collect()
    }
}
