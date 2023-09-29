use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

use crate::{
    db::budget_template::{DynBudgeterConfigRepo, DynExternalExpenseRepo},
    error::DatamizeResult,
    models::budget_template::{BudgetDetails, Budgeter, Configured, MonthTarget},
};

use super::{DynCategoryService, DynScheduledTransactionService};

#[async_trait]
pub trait TemplateDetailServiceExt: DynClone + Send + Sync {
    async fn get_template_details(&mut self, month: MonthTarget) -> DatamizeResult<BudgetDetails>;
}

clone_trait_object!(TemplateDetailServiceExt);

pub type DynTemplateDetailService = Box<dyn TemplateDetailServiceExt>;

#[derive(Clone)]
pub struct TemplateDetailService {
    pub category_service: DynCategoryService,
    pub scheduled_transaction_service: DynScheduledTransactionService,
    pub budgeter_config_repo: DynBudgeterConfigRepo,
    pub external_expense_repo: DynExternalExpenseRepo,
}

impl TemplateDetailService {
    pub fn new_boxed(
        category_service: DynCategoryService,
        scheduled_transaction_service: DynScheduledTransactionService,
        budgeter_config_repo: DynBudgeterConfigRepo,
        external_expense_repo: DynExternalExpenseRepo,
    ) -> Box<Self> {
        Box::new(TemplateDetailService {
            category_service,
            scheduled_transaction_service,
            budgeter_config_repo,
            external_expense_repo,
        })
    }
}

#[async_trait]
impl TemplateDetailServiceExt for TemplateDetailService {
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

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use ynab::{Category, ScheduledTransactionDetail};

    use super::*;
    use crate::{
        db::budget_template::{MockBudgeterConfigRepoImpl, MockExternalExpenseRepoImpl},
        models::budget_template::{DatamizeScheduledTransaction, ExpenseCategorization},
        services::budget_template::{
            category::CategoryServiceExt, scheduled_transaction::ScheduledTransactionServiceExt,
        },
    };

    #[tokio::test]
    async fn get_template_details_should_return_all_scheduled_transactions() {
        #[derive(Clone)]
        struct MockCategoryService {}
        #[async_trait]
        impl CategoryServiceExt for MockCategoryService {
            async fn get_categories_of_month(
                &mut self,
                _month: MonthTarget,
            ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
                Ok((
                    vec![Category {
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
                    }],
                    vec![Faker.fake()],
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

        let mut template_details_service = TemplateDetailService {
            category_service,
            scheduled_transaction_service,
            budgeter_config_repo,
            external_expense_repo,
        };

        template_details_service
            .get_template_details(Faker.fake())
            .await
            .unwrap();
    }
}