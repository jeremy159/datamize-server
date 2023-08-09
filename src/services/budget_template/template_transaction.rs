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

pub struct TemplateTransactionService {
    pub scheduled_transaction_service: Box<dyn ScheduledTransactionServiceExt + Sync + Send>,
    pub ynab_category_repo: Box<dyn YnabCategoryRepo + Sync + Send>,
    pub ynab_client: Arc<dyn CategoryRequests + Sync + Send>,
}

#[async_trait]
impl TemplateTransactionServiceExt for TemplateTransactionService {
    #[tracing::instrument(skip(self))]
    async fn get_template_transactions(
        &mut self,
    ) -> DatamizeResult<ScheduledTransactionsDistribution> {
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

impl TemplateTransactionService {
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

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use mockall::mock;
    use ynab::{
        Category, CategoryGroupWithCategories, CategoryGroupWithCategoriesDelta, SaveMonthCategory,
        ScheduledTransactionDetail, YnabResult,
    };

    use super::*;
    use crate::{
        db::budget_providers::ynab::MockYnabCategoryRepo, error::AppError,
        services::budget_template::scheduled_transaction::MockScheduledTransactionServiceExt,
    };

    mock! {
        YnabClient {}
        #[async_trait]
        impl CategoryRequests for YnabClient {
            async fn get_categories(&self) -> YnabResult<Vec<CategoryGroupWithCategories>>;

            async fn get_categories_delta(
                &self,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<CategoryGroupWithCategoriesDelta>;

            async fn get_category_by_id(&self, category_id: &str) -> YnabResult<Category>;

            async fn get_category_by_id_for(&self, category_id: &str, month: &str) -> YnabResult<Category>;

            async fn update_category_for(
                &self,
                category_id: &str,
                month: &str,
                data: SaveMonthCategory,
            ) -> YnabResult<Category>;
        }
    }

    #[tokio::test]
    async fn get_template_transactions_should_return_all_scheduled_transactions() {
        let mut scheduled_transaction_service = Box::new(MockScheduledTransactionServiceExt::new());
        let mut ynab_category_repo = Box::new(MockYnabCategoryRepo::new());
        let ynab_client = Arc::new(MockYnabClient::new());

        scheduled_transaction_service
            .expect_get_latest_scheduled_transactions()
            .once()
            .returning(|| {
                Ok(vec![Into::into(ScheduledTransactionDetail {
                    id: Faker.fake(),
                    date_first: Faker.fake(),
                    date_next: Faker.fake(),
                    frequency: None,
                    amount: Faker.fake(),
                    memo: Faker.fake(),
                    flag_color: Faker.fake(),
                    account_id: Faker.fake(),
                    payee_id: Faker.fake(),
                    category_id: Faker.fake(),
                    transfer_account_id: Faker.fake(),
                    deleted: Faker.fake(),
                    account_name: Faker.fake(),
                    payee_name: Faker.fake(),
                    category_name: Faker.fake(),
                    subtransactions: vec![],
                })])
            });

        ynab_category_repo.expect_get().returning(|_| {
            Ok(Category {
                id: Faker.fake(),
                category_group_id: Faker.fake(),
                category_group_name: Faker.fake(),
                name: Faker.fake(),
                hidden: false,
                original_category_group_id: None,
                note: Faker.fake(),
                budgeted: Faker.fake(),
                activity: Faker.fake(),
                balance: Faker.fake(),
                goal_type: None,
                goal_day: None,
                goal_cadence: None,
                goal_cadence_frequency: None,
                goal_creation_month: None,
                goal_target: Faker.fake(),
                goal_target_month: None,
                goal_percentage_complete: None,
                goal_months_to_budget: None,
                goal_under_funded: None,
                goal_overall_funded: None,
                goal_overall_left: None,
                deleted: false,
            })
        });

        let mut template_transaction_service = TemplateTransactionService {
            scheduled_transaction_service,
            ynab_category_repo,
            ynab_client,
        };

        template_transaction_service
            .get_template_transactions()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_template_transactions_should_reach_ynab_if_cat_not_in_db() {
        let mut scheduled_transaction_service = Box::new(MockScheduledTransactionServiceExt::new());
        let mut ynab_category_repo = Box::new(MockYnabCategoryRepo::new());
        let mut ynab_client = MockYnabClient::new();

        scheduled_transaction_service
            .expect_get_latest_scheduled_transactions()
            .once()
            .returning(|| {
                Ok(vec![Into::into(ScheduledTransactionDetail {
                    id: Faker.fake(),
                    date_first: Faker.fake(),
                    date_next: Faker.fake(),
                    frequency: None,
                    amount: Faker.fake(),
                    memo: Faker.fake(),
                    flag_color: Faker.fake(),
                    account_id: Faker.fake(),
                    payee_id: Faker.fake(),
                    category_id: Faker.fake(),
                    transfer_account_id: Faker.fake(),
                    deleted: Faker.fake(),
                    account_name: Faker.fake(),
                    payee_name: Faker.fake(),
                    category_name: Faker.fake(),
                    subtransactions: vec![],
                })])
            });

        ynab_category_repo
            .expect_get()
            .returning(|_| Err(AppError::ResourceNotFound));

        ynab_client.expect_get_category_by_id().returning(|_| {
            Ok(Category {
                id: Faker.fake(),
                category_group_id: Faker.fake(),
                category_group_name: Faker.fake(),
                name: Faker.fake(),
                hidden: false,
                original_category_group_id: None,
                note: Faker.fake(),
                budgeted: Faker.fake(),
                activity: Faker.fake(),
                balance: Faker.fake(),
                goal_type: None,
                goal_day: None,
                goal_cadence: None,
                goal_cadence_frequency: None,
                goal_creation_month: None,
                goal_target: Faker.fake(),
                goal_target_month: None,
                goal_percentage_complete: None,
                goal_months_to_budget: None,
                goal_under_funded: None,
                goal_overall_funded: None,
                goal_overall_left: None,
                deleted: false,
            })
        });

        let mut template_transaction_service = TemplateTransactionService {
            scheduled_transaction_service,
            ynab_category_repo,
            ynab_client: Arc::new(ynab_client),
        };

        template_transaction_service
            .get_template_transactions()
            .await
            .unwrap();
    }
}
