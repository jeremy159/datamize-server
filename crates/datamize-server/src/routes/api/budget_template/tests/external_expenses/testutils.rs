use axum::Router;
use datamize_domain::{
    db::{DbResult, ExternalExpenseRepo},
    ExternalExpense,
};
use db_sqlite::budget_template::SqliteExternalExpenseRepo;
use sqlx::SqlitePool;

use crate::{
    routes::api::budget_template::get_external_expense_routes,
    services::budget_template::ExternalExpenseService,
};

pub(crate) struct TestContext {
    external_expense_repo: Box<SqliteExternalExpenseRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let external_expense_repo = SqliteExternalExpenseRepo::new_boxed(pool.clone());

        let external_expense_service =
            ExternalExpenseService::new_arced(external_expense_repo.clone());
        let app = get_external_expense_routes(external_expense_service);
        Self {
            external_expense_repo,
            app,
        }
    }

    pub(crate) fn app(&self) -> Router {
        self.app.clone()
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
    }

    pub(crate) async fn set_external_expenses(&self, external_expenses: &[ExternalExpense]) {
        for ee in external_expenses {
            self.external_expense_repo.update(ee).await.unwrap();
        }
    }

    pub(crate) async fn get_external_expense_by_name(
        &self,
        external_expense_name: &str,
    ) -> DbResult<ExternalExpense> {
        self.external_expense_repo
            .get_by_name(external_expense_name)
            .await
    }
}
