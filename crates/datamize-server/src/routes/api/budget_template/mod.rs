mod budgeter;
mod budgeters;
mod details;
mod expense_categorization;
mod expenses_categorization;
mod summary;
#[cfg(test)]
mod tests;
mod transactions;

use axum::{
    routing::{get, post},
    Router,
};
use budgeter::*;
use budgeters::*;
use db_postgres::{
    budget_providers::ynab::{PostgresYnabCategoryRepo, PostgresYnabScheduledTransactionRepo},
    budget_template::{PostgresBudgeterConfigRepo, PostgresExpenseCategorizationRepo},
};
use db_redis::budget_providers::ynab::{
    RedisYnabCategoryMetaRepo, RedisYnabScheduledTransactionMetaRepo,
};
use details::*;
use expense_categorization::*;
use expenses_categorization::*;
use summary::*;
use transactions::*;

use crate::{
    services::budget_template::{
        BudgeterService, CategoryService, DynBudgeterService, DynExpenseCategorizationService,
        DynTemplateDetailService, DynTemplateSummaryService, DynTemplateTransactionService,
        ExpenseCategorizationService, ScheduledTransactionService, TemplateDetailService,
        TemplateSummaryService, TemplateTransactionService,
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

    let expense_categorization_service =
        ExpenseCategorizationService::new_arced(expense_categorization_repo);

    Router::new()
        .merge(get_detail_routes(template_detail_service))
        .merge(get_summary_routes(template_summary_service))
        .merge(get_transaction_routes(template_transaction_service))
        .merge(get_budgeter_routes(budgeter_service))
        .merge(get_expense_categorization_routes(
            expense_categorization_service,
        ))
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

fn get_budgeter_routes<S>(budgeter_service: DynBudgeterService) -> Router<S> {
    Router::new()
        .route("/budgeters", get(get_all_budgeters))
        .route("/budgeter", post(create_budgeter))
        .route(
            "/budgeter/:budgeter_id",
            get(get_budgeter)
                .put(update_budgeter)
                .delete(delete_budgeter),
        )
        .with_state(budgeter_service)
}

fn get_expense_categorization_routes<S>(
    expense_categorization_service: DynExpenseCategorizationService,
) -> Router<S> {
    Router::new()
        .route(
            "/expenses_categorization",
            get(get_all_expenses_categorization).put(update_all_expenses_categorization),
        )
        .route(
            "/expense_categorization/:expense_categorization_id",
            get(get_expense_categorization).put(update_expense_categorization),
        )
        .with_state(expense_categorization_service)
}
