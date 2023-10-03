use datamize_domain::{
    async_trait,
    db::{DynBudgeterConfigRepo, DynExternalExpenseRepo},
    BudgetDetails, BudgetSummary, Budgeter, Configured, MonthTarget,
};
use dyn_clone::{clone_trait_object, DynClone};

use crate::error::DatamizeResult;

use super::{DynCategoryService, DynScheduledTransactionService};

#[async_trait]
pub trait TemplateSummaryServiceExt: DynClone + Send + Sync {
    async fn get_template_summary(&mut self, month: MonthTarget) -> DatamizeResult<BudgetSummary>;
}

clone_trait_object!(TemplateSummaryServiceExt);

pub type DynTemplateSummaryService = Box<dyn TemplateSummaryServiceExt>;

#[derive(Clone)]
pub struct TemplateSummaryService {
    pub category_service: DynCategoryService,
    pub scheduled_transaction_service: DynScheduledTransactionService,
    pub budgeter_config_repo: DynBudgeterConfigRepo,
    pub external_expense_repo: DynExternalExpenseRepo,
}

impl TemplateSummaryService {
    pub fn new_boxed(
        category_service: DynCategoryService,
        scheduled_transaction_service: DynScheduledTransactionService,
        budgeter_config_repo: DynBudgeterConfigRepo,
        external_expense_repo: DynExternalExpenseRepo,
    ) -> Box<Self> {
        Box::new(TemplateSummaryService {
            category_service,
            scheduled_transaction_service,
            budgeter_config_repo,
            external_expense_repo,
        })
    }
}

#[async_trait]
impl TemplateSummaryServiceExt for TemplateSummaryService {
    #[tracing::instrument(skip(self))]
    async fn get_template_summary(&mut self, month: MonthTarget) -> DatamizeResult<BudgetSummary> {
        let (saved_categories, expenses_categorization) =
            self.category_service.get_categories_of_month(month).await?;
        let saved_scheduled_transactions = self
            .scheduled_transaction_service
            .get_latest_scheduled_transactions()
            .await?;
        let external_expenses = self.external_expense_repo.get_all().await?;
        let budgeters_config = self.budgeter_config_repo.get_all().await?;
        let budgeters: Vec<_> = budgeters_config
            .into_iter()
            .map(|bc| {
                Budgeter::<Configured>::from(bc).compute_salary(&saved_scheduled_transactions)
            })
            .collect();

        let budget_details = BudgetDetails::build(
            saved_categories,
            saved_scheduled_transactions,
            &month.into(),
            external_expenses,
            expenses_categorization,
            &budgeters,
        );

        Ok(BudgetSummary::build(&budget_details, budgeters))
    }
}

#[cfg(test)]
mod tests {
    use datamize_domain::{
        db::{MockBudgeterConfigRepoImpl, MockExternalExpenseRepoImpl},
        DatamizeScheduledTransaction, ExpenseCategorization,
    };
    use fake::{Fake, Faker};
    use ynab::Category;

    use super::*;
    use crate::services::budget_template::{
        category::CategoryServiceExt, scheduled_transaction::ScheduledTransactionServiceExt,
    };

    #[tokio::test]
    async fn get_template_summary_should_return_all_scheduled_transactions() {
        #[derive(Clone)]
        struct MockCategoryService {}
        #[async_trait]
        impl CategoryServiceExt for MockCategoryService {
            async fn get_categories_of_month(
                &mut self,
                _month: MonthTarget,
            ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
                Ok((
                    fake::vec![Category; 1..5],
                    fake::vec![ExpenseCategorization; 1..3],
                ))
            }
        }

        let category_service = Box::new(MockCategoryService {});
        #[derive(Clone)]
        struct MockScheduledTransactionService {}
        #[async_trait]
        impl ScheduledTransactionServiceExt for MockScheduledTransactionService {
            async fn get_latest_scheduled_transactions(
                &mut self,
            ) -> DatamizeResult<Vec<DatamizeScheduledTransaction>> {
                Ok(fake::vec![DatamizeScheduledTransaction; 1..5])
            }
        }

        let scheduled_transaction_service = Box::new(MockScheduledTransactionService {});
        let mut budgeter_config_repo = Box::new(MockBudgeterConfigRepoImpl::new());
        let mut external_expense_repo = Box::new(MockExternalExpenseRepoImpl::new());

        external_expense_repo
            .expect_get_all()
            .return_once(|| Ok(vec![Faker.fake(), Faker.fake()]));

        budgeter_config_repo
            .expect_get_all()
            .return_once(|| Ok(vec![Faker.fake(), Faker.fake()]));

        let mut template_summary_service = TemplateSummaryService {
            category_service,
            scheduled_transaction_service,
            budgeter_config_repo,
            external_expense_repo,
        };

        template_summary_service
            .get_template_summary(Faker.fake())
            .await
            .unwrap();
    }
}
