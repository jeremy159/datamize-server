use std::sync::Arc;

use async_trait::async_trait;
use datamize_domain::{
    db::{
        ynab::{DynYnabCategoryMetaRepo, DynYnabCategoryRepo, YnabCategoryMetaRepo},
        DbResult, DynExpenseCategorizationRepo,
    },
    ExpenseCategorization,
};
use db_redis::{budget_providers::ynab::RedisYnabCategoryMetaRepo, get_test_pool};
use db_sqlite::{
    budget_providers::ynab::SqliteYnabCategoryRepo,
    budget_template::SqliteExpenseCategorizationRepo,
};
use fake::{Fake, Faker};
use sqlx::SqlitePool;
use ynab::{
    Category, CategoryGroupWithCategories, CategoryGroupWithCategoriesDelta, CategoryRequests,
    MonthDetail, MonthRequests, MonthSummary, MonthSummaryDelta, SaveMonthCategory, YnabResult,
};

use crate::services::budget_providers::CategoryService;

pub(crate) struct TestContext {
    ynab_category_repo: DynYnabCategoryRepo,
    ynab_category_meta_repo: DynYnabCategoryMetaRepo,
    expense_categorization_repo: DynExpenseCategorizationRepo,
    category_service: CategoryService<MockMonthAndCategoriesRequestsImpl>,
}

impl TestContext {
    pub(crate) async fn setup(
        pool: SqlitePool,
        ynab_categories: CategoryGroupWithCategoriesDelta,
        ynab_month: MonthDetail,
    ) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let ynab_category_repo = SqliteYnabCategoryRepo::new_arced(pool.clone());
        let ynab_category_meta_repo = RedisYnabCategoryMetaRepo::new_arced(redis_conn_pool);
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
            .returning(move |_| Ok(ynab_month.clone()));

        let category_service = CategoryService {
            ynab_category_repo: ynab_category_repo.clone(),
            ynab_category_meta_repo: ynab_category_meta_repo.clone(),
            expense_categorization_repo: expense_categorization_repo.clone(),
            ynab_client,
        };

        Self {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            category_service,
        }
    }

    pub(crate) fn service(&self) -> &CategoryService<MockMonthAndCategoriesRequestsImpl> {
        &self.category_service
    }

    pub(crate) async fn set_categories(&self, categories: &[Category]) {
        self.ynab_category_repo
            .update_all(categories)
            .await
            .unwrap();
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

    pub(crate) async fn get_delta(&self) -> DbResult<i64> {
        self.ynab_category_meta_repo.get_delta().await
    }

    pub(crate) async fn get_last_saved(&self) -> String {
        self.ynab_category_meta_repo.get_last_saved().await.unwrap()
    }

    pub(crate) async fn set_last_saved(&self, date: String) {
        self.ynab_category_meta_repo
            .set_last_saved(date)
            .await
            .unwrap()
    }
}

mockall::mock! {
    pub MonthAndCategoriesRequestsImpl {}

    impl Clone for MonthAndCategoriesRequestsImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl MonthRequests for MonthAndCategoriesRequestsImpl {
        async fn get_months(&self) -> YnabResult<Vec<MonthSummary> > ;
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
