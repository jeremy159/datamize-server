use std::sync::Arc;

use datamize_domain::{
    db::{DbResult, ExpenseCategorizationRepo},
    ExpenseCategorization, Uuid,
};
use db_sqlite::budget_template::SqliteExpenseCategorizationRepo;
use sqlx::SqlitePool;

use crate::services::budget_template::{
    DynExpenseCategorizationService, ExpenseCategorizationService, ExpenseCategorizationServiceExt,
};

pub(crate) struct TestContext {
    expense_categorization_repo: Arc<SqliteExpenseCategorizationRepo>,
    expense_categorization_service: DynExpenseCategorizationService,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let expense_categorization_repo = SqliteExpenseCategorizationRepo::new_arced(pool.clone());

        let expense_categorization_service =
            ExpenseCategorizationService::new_arced(expense_categorization_repo.clone());

        Self {
            expense_categorization_repo,
            expense_categorization_service,
        }
    }

    pub(crate) fn service(&self) -> &dyn ExpenseCategorizationServiceExt {
        self.expense_categorization_service.as_ref()
    }

    pub(crate) fn into_service(self) -> DynExpenseCategorizationService {
        self.expense_categorization_service
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
