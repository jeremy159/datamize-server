mod budgeter;
mod budgeters;
mod details;
mod expense_categorization;
mod expenses_categorization;
mod external_expense;
mod external_expenses;
mod summary;
mod transactions;

use axum::{
    routing::{get, post},
    Router,
};
use budgeter::*;
use budgeters::*;
use details::*;
use expense_categorization::*;
use expenses_categorization::*;
use external_expense::*;
use external_expenses::*;
use summary::*;
use transactions::*;

use crate::{
    services::budget_template::{
        BudgeterService, DynBudgeterService, DynExpenseCategorizationService,
        DynExternalExpenseService, DynTemplateDetailService, DynTemplateSummaryService,
        DynTemplateTransactionService, ExpenseCategorizationService, ExternalExpenseService,
        TemplateDetailService, TemplateSummaryService, TemplateTransactionService,
    },
    startup::AppState,
};

pub fn get_budget_template_routes<S: Clone + Send + Sync + 'static>(
    app_state: &AppState,
) -> Router<S> {
    let template_detail_service = TemplateDetailService::new_boxed(
        app_state.db_conn_pool.clone(),
        app_state.redis_conn.clone(),
        app_state.ynab_client.clone(),
    );

    let template_summary_service = TemplateSummaryService::new_boxed(
        app_state.db_conn_pool.clone(),
        app_state.redis_conn.clone(),
        app_state.ynab_client.clone(),
    );

    let template_transaction_service = TemplateTransactionService::new_boxed(
        app_state.db_conn_pool.clone(),
        app_state.redis_conn.clone(),
        app_state.ynab_client.clone(),
    );

    let budgeter_service = BudgeterService::new_arced(app_state.db_conn_pool.clone());

    let external_expense_service =
        ExternalExpenseService::new_arced(app_state.db_conn_pool.clone());

    let expense_categorization_service =
        ExpenseCategorizationService::new_arced(app_state.db_conn_pool.clone());

    Router::new()
        .merge(get_detail_routes(template_detail_service))
        .merge(get_summary_routes(template_summary_service))
        .merge(get_transaction_routes(template_transaction_service))
        .merge(get_budgeter_routes(budgeter_service))
        .merge(get_external_expense_routes(external_expense_service))
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

fn get_external_expense_routes<S>(
    external_expense_service: DynExternalExpenseService,
) -> Router<S> {
    Router::new()
        .route("/external_expenses", get(get_all_external_expenses))
        .route("/external_expense", post(create_external_expense))
        .route(
            "/external_expense/:external_expense_id",
            get(get_external_expense)
                .put(update_external_expense)
                .delete(delete_external_expense),
        )
        .with_state(external_expense_service)
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
