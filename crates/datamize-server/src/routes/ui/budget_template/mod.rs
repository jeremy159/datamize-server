mod budgeter;
mod details;
mod summary;
#[cfg(test)]
mod tests;
mod transactions;

use axum::{routing::get, Router};
use db_postgres::{
    budget_providers::ynab::{
        PostgresYnabCategoryRepo, PostgresYnabPayeeRepo, PostgresYnabScheduledTransactionRepo,
    },
    budget_template::{PostgresBudgeterConfigRepo, PostgresExpenseCategorizationRepo},
};
use db_redis::budget_providers::ynab::{
    RedisYnabCategoryMetaRepo, RedisYnabPayeeMetaRepo, RedisYnabScheduledTransactionMetaRepo,
};
use details::*;
use summary::*;
use transactions::*;

use crate::{
    services::{
        budget_providers::{
            CategoryService, DynYnabPayeeService, ScheduledTransactionService, YnabPayeeService,
        },
        budget_template::{
            BudgeterService, DynBudgeterService, DynExpenseCategorizationService,
            DynTemplateDetailService, DynTemplateSummaryService, DynTemplateTransactionService,
            ExpenseCategorizationService, TemplateDetailService, TemplateSummaryService,
            TemplateTransactionService,
        },
    },
    startup::AppState,
};

pub fn get_budget_template_routes<S: Clone + Send + Sync + 'static>(
    app_state: &AppState,
) -> Router<S> {
    let ynab_category_repo = PostgresYnabCategoryRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_category_meta_repo =
        RedisYnabCategoryMetaRepo::new_arced(app_state.redis_conn_pool.clone());
    let ynab_scheduled_transaction_repo =
        PostgresYnabScheduledTransactionRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_scheduled_transaction_meta_repo =
        RedisYnabScheduledTransactionMetaRepo::new_arced(app_state.redis_conn_pool.clone());
    let expense_categorization_repo =
        PostgresExpenseCategorizationRepo::new_arced(app_state.db_conn_pool.clone());
    let budgeter_config_repo =
        PostgresBudgeterConfigRepo::new_arced(app_state.db_conn_pool.clone());
    let category_service = CategoryService::new_arced(
        ynab_category_repo.clone(),
        ynab_category_meta_repo,
        expense_categorization_repo.clone(),
        app_state.ynab_client.clone(),
    );
    let scheduled_transaction_service = ScheduledTransactionService::new_arced(
        ynab_scheduled_transaction_repo,
        ynab_scheduled_transaction_meta_repo,
        app_state.ynab_client.clone(),
    );

    let template_detail_service = TemplateDetailService::new_arced(
        category_service.clone(),
        scheduled_transaction_service.clone(),
        budgeter_config_repo.clone(),
    );

    let template_summary_service = TemplateSummaryService::new_arced(
        category_service,
        scheduled_transaction_service.clone(),
        budgeter_config_repo.clone(),
    );

    let template_transaction_service = TemplateTransactionService::new_arced(
        scheduled_transaction_service,
        ynab_category_repo,
        app_state.ynab_client.clone(),
    );

    let budgeter_service = BudgeterService::new_arced(budgeter_config_repo);

    let ynab_payee_repo = PostgresYnabPayeeRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_payee_meta_repo = RedisYnabPayeeMetaRepo::new_arced(app_state.redis_conn_pool.clone());
    let ynab_payee_service = YnabPayeeService::new_arced(
        ynab_payee_repo,
        ynab_payee_meta_repo,
        app_state.ynab_client.clone(),
    );

    let expense_categorization_service =
        ExpenseCategorizationService::new_arced(expense_categorization_repo);

    Router::new()
        .merge(get_detail_routes(template_detail_service))
        .merge(get_summary_routes(template_summary_service))
        .merge(get_transaction_routes(template_transaction_service))
        .merge(get_budgeter_routes(budgeter_service, ynab_payee_service))
}

fn get_detail_routes<S>(template_detail_service: DynTemplateDetailService) -> Router<S> {
    Router::new()
        .route("/details", get(template_details))
        .with_state(template_detail_service)
}

fn get_summary_routes<S>(template_summary_service: DynTemplateSummaryService) -> Router<S> {
    Router::new()
        .route("/summary", get(template_summary))
        .with_state(template_summary_service)
}

fn get_transaction_routes<S>(
    template_transaction_service: DynTemplateTransactionService,
) -> Router<S> {
    Router::new()
        .route("/transactions", get(template_transactions))
        .with_state(template_transaction_service)
}

fn get_budgeter_routes<S>(
    budgeter_service: DynBudgeterService,
    ynab_payee_service: DynYnabPayeeService,
) -> Router<S> {
    Router::new()
        .route(
            "/budgeter/new",
            get(budgeter::new::get).post(budgeter::new::post),
        )
        .route(
            "/budgeter/:budgeter_id",
            get(budgeter::edit::get)
                .post(budgeter::edit::post)
                .delete(budgeter::delete::delete),
        )
        .with_state((budgeter_service, ynab_payee_service))
}
