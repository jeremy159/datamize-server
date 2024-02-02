use std::sync::Arc;

use async_trait::async_trait;
use datamize_domain::{
    db::{
        ynab::{YnabCategoryMetaRepo, YnabScheduledTransactionMetaRepo},
        BudgeterConfigRepo, ExpenseCategorizationRepo,
    },
    BudgeterConfig, ExpenseCategorization,
};
use db_redis::{
    budget_providers::ynab::{RedisYnabCategoryMetaRepo, RedisYnabScheduledTransactionMetaRepo},
    get_test_pool,
};
use db_sqlite::{
    budget_providers::ynab::{SqliteYnabCategoryRepo, SqliteYnabScheduledTransactionRepo},
    budget_template::{SqliteBudgeterConfigRepo, SqliteExpenseCategorizationRepo},
};
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{
    Category, CategoryGroupWithCategories, CategoryGroupWithCategoriesDelta, CategoryRequests,
    MockScheduledTransactionRequestsImpl, MonthDetail, MonthRequests, MonthSummary,
    MonthSummaryDelta, SaveMonthCategory, ScheduledTransactionsDetailDelta, YnabResult,
};

use crate::services::{
    budget_providers::{CategoryService, ScheduledTransactionService},
    budget_template::{DynTemplateDetailService, TemplateDetailService},
};

pub(crate) struct TestContext {
    budgeter_config_repo: Arc<SqliteBudgeterConfigRepo>,
    expense_categorization_repo: Arc<SqliteExpenseCategorizationRepo>,
    template_detail_service: DynTemplateDetailService,
}

impl TestContext {
    pub(crate) async fn setup(
        pool: SqlitePool,
        ynab_categories: CategoryGroupWithCategoriesDelta,
        ynab_scheduled_transactions: ScheduledTransactionsDetailDelta,
    ) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let budgeter_config_repo = SqliteBudgeterConfigRepo::new_arced(pool.clone());
        let ynab_category_repo = SqliteYnabCategoryRepo::new_arced(pool.clone());
        let ynab_category_meta_repo = RedisYnabCategoryMetaRepo::new_arced(redis_conn_pool.clone());
        ynab_category_meta_repo
            .set_delta(Faker.fake())
            .await
            .unwrap();
        let expense_categorization_repo = SqliteExpenseCategorizationRepo::new_arced(pool.clone());
        let mut ynab_client = Arc::new(MockMonthAndCategoriesRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_categories_delta()
            .returning(move |_| Ok(ynab_categories.clone()));
        ynab_client_mock
            .expect_get_month_by_date()
            .returning(|_| Ok(Faker.fake()));
        let category_service = CategoryService::new_arced(
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo.clone(),
            ynab_client,
        );
        let ynab_scheduled_transaction_repo =
            SqliteYnabScheduledTransactionRepo::new_arced(pool.clone());
        let ynab_scheduled_transaction_meta_repo =
            RedisYnabScheduledTransactionMetaRepo::new_arced(redis_conn_pool);
        ynab_scheduled_transaction_meta_repo
            .set_delta(Faker.fake())
            .await
            .unwrap();
        let mut ynab_client = Arc::new(MockScheduledTransactionRequestsImpl::new());
        let ynab_client_mock = Arc::make_mut(&mut ynab_client);
        ynab_client_mock
            .expect_get_scheduled_transactions_delta()
            .returning(move |_| Ok(ynab_scheduled_transactions.clone()));
        let scheduled_transaction_service = ScheduledTransactionService::new_arced(
            ynab_scheduled_transaction_repo,
            ynab_scheduled_transaction_meta_repo,
            ynab_client,
        );

        let template_detail_service = TemplateDetailService::new_arced(
            category_service,
            scheduled_transaction_service,
            budgeter_config_repo.clone(),
        );

        Self {
            budgeter_config_repo,
            expense_categorization_repo,
            template_detail_service,
        }
    }

    pub(crate) fn into_service(self) -> DynTemplateDetailService {
        self.template_detail_service
    }

    pub(crate) async fn set_budgeters(&self, budgeters: &[BudgeterConfig]) {
        for b in budgeters {
            self.budgeter_config_repo.update(b).await.unwrap();
        }
    }

    pub(crate) async fn set_expenses_categorization(
        &self,
        expenses_categorization: &[ExpenseCategorization],
    ) {
        self.expense_categorization_repo
            .update_all(expenses_categorization)
            .await
            .unwrap();
    }
}

mockall::mock! {
    pub MonthAndCategoriesRequestsImpl {}

    impl Clone for MonthAndCategoriesRequestsImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl MonthRequests for MonthAndCategoriesRequestsImpl {
        async fn get_months(&self) -> YnabResult<Vec<MonthSummary>>;
        async fn get_months_delta(
            &self,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<MonthSummaryDelta>;
        async fn get_month_by_date(&self, date: &str) -> YnabResult<MonthDetail>;
    }

    #[async_trait]
    impl CategoryRequests for MonthAndCategoriesRequestsImpl {
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
