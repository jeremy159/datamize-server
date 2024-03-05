use std::sync::Arc;

use datamize_domain::{
    async_trait, db::DynBudgeterConfigRepo, BudgetDetails, Budgeter, Configured, MonthTarget,
};

use crate::{
    error::DatamizeResult,
    services::budget_providers::{DynCategoryService, DynScheduledTransactionService},
};

#[async_trait]
pub trait TemplateDetailServiceExt: Send + Sync {
    async fn get_template_details(
        &self,
        month: MonthTarget,
        use_category_groups_as_sub_type: bool,
    ) -> DatamizeResult<BudgetDetails>;
}

pub type DynTemplateDetailService = Arc<dyn TemplateDetailServiceExt>;

#[derive(Clone)]
pub struct TemplateDetailService {
    pub category_service: DynCategoryService,
    pub scheduled_transaction_service: DynScheduledTransactionService,
    pub budgeter_config_repo: DynBudgeterConfigRepo,
}

impl TemplateDetailService {
    pub fn new_arced(
        category_service: DynCategoryService,
        scheduled_transaction_service: DynScheduledTransactionService,
        budgeter_config_repo: DynBudgeterConfigRepo,
    ) -> Arc<Self> {
        Arc::new(TemplateDetailService {
            category_service,
            scheduled_transaction_service,
            budgeter_config_repo,
        })
    }
}

#[async_trait]
impl TemplateDetailServiceExt for TemplateDetailService {
    #[tracing::instrument(skip(self))]
    async fn get_template_details(
        &self,
        month: MonthTarget,
        use_category_groups_as_sub_type: bool,
    ) -> DatamizeResult<BudgetDetails> {
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

        Ok(BudgetDetails::build(
            saved_categories,
            saved_scheduled_transactions,
            &month.into(),
            expenses_categorization,
            &budgeters,
            use_category_groups_as_sub_type,
        ))
    }
}
