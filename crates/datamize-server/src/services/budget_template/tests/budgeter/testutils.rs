use std::sync::Arc;

use datamize_domain::{
    db::{BudgeterConfigRepo, DbResult},
    BudgeterConfig,
};
use db_sqlite::budget_template::SqliteBudgeterConfigRepo;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    services::budget_template::{BudgeterService, BudgeterServiceExt, DynBudgeterService},
};

pub(crate) enum ErrorType {
    Internal,
    NotFound,
    AlreadyExist,
}

pub(crate) struct TestContext {
    budgeter_config_repo: Arc<SqliteBudgeterConfigRepo>,
    budgeter_service: DynBudgeterService,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let budgeter_config_repo = SqliteBudgeterConfigRepo::new_arced(pool.clone());

        let budgeter_service = BudgeterService::new_arced(budgeter_config_repo.clone());

        Self {
            budgeter_config_repo,
            budgeter_service,
        }
    }

    pub(crate) fn service(&self) -> &dyn BudgeterServiceExt {
        self.budgeter_service.as_ref()
    }

    pub(crate) fn into_service(self) -> DynBudgeterService {
        self.budgeter_service
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

pub(crate) fn assert_err(err: AppError, expected_err: Option<ErrorType>) {
    match expected_err {
        Some(ErrorType::Internal) => assert!(matches!(err, AppError::InternalServerError(_))),
        Some(ErrorType::NotFound) => {
            assert!(matches!(err, AppError::ResourceNotFound))
        }
        Some(ErrorType::AlreadyExist) => assert!(matches!(err, AppError::ResourceAlreadyExist)),
        None => {
            // noop
        }
    }
}
