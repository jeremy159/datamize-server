use std::sync::Arc;

use async_trait::async_trait;
use ynab::Client;

use crate::{
    db::budget_template::{BudgeterConfigRepo, ExternalExpenseRepo},
    error::DatamizeResult,
    models::budget_template::{BudgetDetails, Budgeter, Configured, MonthTarget},
};

use super::{CategoryServiceExt, ScheduledTransactionServiceExt};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TemplateDetailServiceExt {
    async fn get_template_details(&mut self, month: MonthTarget) -> DatamizeResult<BudgetDetails>;
}

pub struct TemplateDetailService<BCR: BudgeterConfigRepo, EER: ExternalExpenseRepo> {
    pub category_service: Box<dyn CategoryServiceExt + Sync + Send>,
    pub scheduled_transaction_service: Box<dyn ScheduledTransactionServiceExt + Sync + Send>,
    pub budgeter_config_repo: BCR,
    pub external_expense_repo: EER,
    pub ynab_client: Arc<Client>,
}

#[async_trait]
impl<BCR, EER> TemplateDetailServiceExt for TemplateDetailService<BCR, EER>
where
    BCR: BudgeterConfigRepo + Sync + Send,
    EER: ExternalExpenseRepo + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_template_details(&mut self, month: MonthTarget) -> DatamizeResult<BudgetDetails> {
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

        Ok(BudgetDetails::build(
            saved_categories,
            saved_scheduled_transactions,
            &month.into(),
            external_expenses,
            expenses_categorization,
            &budgeters,
        ))
    }
}
