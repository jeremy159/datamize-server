use std::sync::Arc;

use datamize_domain::{
    async_trait, db::DynBudgeterConfigRepo, BudgetDetails, BudgetSummary, Budgeter, Configured,
    MonthTarget,
};

use crate::error::DatamizeResult;

use super::{DynCategoryService, DynScheduledTransactionService};

#[async_trait]
pub trait TemplateSummaryServiceExt: Send + Sync {
    async fn get_template_summary(&self, month: MonthTarget) -> DatamizeResult<BudgetSummary>;
}

pub type DynTemplateSummaryService = Arc<dyn TemplateSummaryServiceExt>;

#[derive(Clone)]
pub struct TemplateSummaryService {
    pub category_service: DynCategoryService,
    pub scheduled_transaction_service: DynScheduledTransactionService,
    pub budgeter_config_repo: DynBudgeterConfigRepo,
}

impl TemplateSummaryService {
    pub fn new_arced(
        category_service: DynCategoryService,
        scheduled_transaction_service: DynScheduledTransactionService,
        budgeter_config_repo: DynBudgeterConfigRepo,
    ) -> Arc<Self> {
        Arc::new(TemplateSummaryService {
            category_service,
            scheduled_transaction_service,
            budgeter_config_repo,
        })
    }
}

#[async_trait]
impl TemplateSummaryServiceExt for TemplateSummaryService {
    #[tracing::instrument(skip(self))]
    async fn get_template_summary(&self, month: MonthTarget) -> DatamizeResult<BudgetSummary> {
        let (saved_categories, expenses_categorization) =
            self.category_service.get_categories_of_month(month).await?;
        let saved_scheduled_transactions = self
            .scheduled_transaction_service
            .get_latest_scheduled_transactions()
            .await?;

        let inflow_cat_id = saved_categories
            .iter()
            .find(|c| c.name.contains("Ready to Assign"))
            .map(|c| c.id);
        let budgeters_config = self.budgeter_config_repo.get_all().await?;
        let budgeters: Vec<_> = budgeters_config
            .into_iter()
            .map(|bc| {
                Budgeter::<Configured>::from(bc).compute_salary(
                    &saved_scheduled_transactions,
                    &month.into(),
                    inflow_cat_id,
                )
            })
            .collect();

        let budget_details = BudgetDetails::build(
            saved_categories,
            saved_scheduled_transactions,
            &month.into(),
            expenses_categorization,
            &budgeters,
        );

        Ok(BudgetSummary::build(&budget_details, budgeters))
    }
}

// #[cfg(test)]
// mod tests {
//     use datamize_domain::{
//         db::MockBudgeterConfigRepo, DatamizeScheduledTransaction, ExpenseCategorization,
//     };
//     use fake::{Fake, Faker};
//     use ynab::Category;

//     use super::*;
//     use crate::services::budget_template::{
//         category::CategoryServiceExt, scheduled_transaction::ScheduledTransactionServiceExt,
//     };

//     #[tokio::test]
//     async fn get_template_summary_should_return_all_scheduled_transactions() {
//         #[derive(Clone)]
//         struct MockCategoryService {}
//         #[async_trait]
//         impl CategoryServiceExt for MockCategoryService {
//             async fn get_categories_of_month(
//                 &self,
//                 _month: MonthTarget,
//             ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
//                 Ok((
//                     fake::vec![Category; 1..5],
//                     fake::vec![ExpenseCategorization; 1..3],
//                 ))
//             }
//         }

//         let category_service = Arc::new(MockCategoryService {});
//         #[derive(Clone)]
//         struct MockScheduledTransactionService {}
//         #[async_trait]
//         impl ScheduledTransactionServiceExt for MockScheduledTransactionService {
//             async fn get_latest_scheduled_transactions(
//                 &self,
//             ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>> {
//                 Ok(fake::vec![DatamizeScheduledTransaction; 1..5])
//             }
//         }

//         let scheduled_transaction_service = Arc::new(MockScheduledTransactionService {});
//         let mut budgeter_config_repo = Arc::new(MockBudgeterConfigRepo::new());

//         budgeter_config_repo
//             .expect_get_all()
//             .return_once(|| Ok(vec![Faker.fake(), Faker.fake()]));

//         let mut template_summary_service = TemplateSummaryService {
//             category_service,
//             scheduled_transaction_service,
//             budgeter_config_repo,
//         };

//         template_summary_service
//             .get_template_summary(Faker.fake())
//             .await
//             .unwrap();
//     }
// }
