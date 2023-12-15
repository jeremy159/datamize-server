use axum::Router;
use datamize_domain::{
    db::{DbResult, ExpenseCategorizationRepo},
    ExpenseCategorization, Uuid,
};
use db_sqlite::budget_template::SqliteExpenseCategorizationRepo;
use sqlx::SqlitePool;

use crate::{
    routes::api::budget_template::get_expense_categorization_routes,
    services::budget_template::ExpenseCategorizationService,
};

pub(crate) struct TestContext {
    expense_categorization_repo: Box<SqliteExpenseCategorizationRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let expense_categorization_repo = SqliteExpenseCategorizationRepo::new_boxed(pool.clone());

        let expense_categorization_service =
            ExpenseCategorizationService::new_arced(expense_categorization_repo.clone());
        let app = get_expense_categorization_routes(expense_categorization_service);
        Self {
            expense_categorization_repo,
            app,
        }
    }

    pub(crate) fn app(&self) -> Router {
        self.app.clone()
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
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

    pub(crate) async fn get_all_expenses_categorization(
        &self,
    ) -> DbResult<Vec<ExpenseCategorization>> {
        self.expense_categorization_repo.get_all().await
    }

    pub(crate) async fn get_expense_categorization(
        &self,
        id: Uuid,
    ) -> DbResult<ExpenseCategorization> {
        self.expense_categorization_repo.get(id).await
    }
}
