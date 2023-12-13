use axum::Router;
use datamize_domain::{
    db::{BudgeterConfigRepo, DbResult},
    BudgeterConfig,
};
use db_sqlite::budget_template::SqliteBudgeterConfigRepo;
use sqlx::SqlitePool;

use crate::{
    routes::api::budget_template::get_budgeter_routes, services::budget_template::BudgeterService,
};

pub(crate) struct TestContext {
    budgeter_config_repo: Box<SqliteBudgeterConfigRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let budgeter_config_repo = SqliteBudgeterConfigRepo::new_boxed(pool.clone());

        let budgeter_service = BudgeterService::new_arced(budgeter_config_repo.clone());
        let app = get_budgeter_routes(budgeter_service);
        Self {
            budgeter_config_repo,
            app,
        }
    }

    pub(crate) fn app(&self) -> Router {
        self.app.clone()
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
    }

    pub(crate) async fn set_budgeters(&self, budgeters: &[BudgeterConfig]) {
        for b in budgeters {
            self.budgeter_config_repo.update(b).await.unwrap();
        }
    }

    pub(crate) async fn get_budgeter_by_name(
        &self,
        budgeter_name: &str,
    ) -> DbResult<BudgeterConfig> {
        self.budgeter_config_repo.get_by_name(budgeter_name).await
    }
}
