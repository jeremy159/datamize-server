use async_trait::async_trait;

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

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use ynab::{Category, ScheduledTransactionDetail};

    use super::*;
    use crate::{
        db::budget_template::{MockBudgeterConfigRepo, MockExternalExpenseRepo},
        services::budget_template::{
            category::MockCategoryServiceExt,
            scheduled_transaction::MockScheduledTransactionServiceExt,
        },
    };

    #[tokio::test]
    async fn get_template_details_should_return_all_scheduled_transactions() {
        let mut category_service = MockCategoryServiceExt::new();
        let mut scheduled_transaction_service = MockScheduledTransactionServiceExt::new();
        let mut budgeter_config_repo = MockBudgeterConfigRepo::new();
        let mut external_expense_repo = MockExternalExpenseRepo::new();

        category_service
            .expect_get_categories_of_month()
            .once()
            .returning(|_| {
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
            });

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

        external_expense_repo
            .expect_get_all()
            .return_once(|| Ok(vec![Faker.fake(), Faker.fake()]));

        budgeter_config_repo
            .expect_get_all()
            .return_once(|| Ok(vec![Faker.fake(), Faker.fake()]));

        let mut template_details_service = TemplateDetailService {
            category_service: Box::new(category_service),
            scheduled_transaction_service: Box::new(scheduled_transaction_service),
            budgeter_config_repo,
            external_expense_repo,
        };

        template_details_service
            .get_template_details(Faker.fake())
            .await
            .unwrap();
    }
}
