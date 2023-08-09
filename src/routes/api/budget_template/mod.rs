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
    extract::FromRef,
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
    db::{
        budget_providers::ynab::{
            PostgresYnabCategoryRepo, PostgresYnabScheduledTransactionRepo,
            RedisYnabCategoryMetaRepo, RedisYnabScheduledTransactionMetaRepo,
        },
        budget_template::{
            PostgresBudgeterConfigRepo, PostgresExpenseCategorizationRepo,
            PostgresExternalExpenseRepo,
        },
    },
    services::budget_template::{
        BudgeterService, CategoryService, ExpenseCategorizationService, ExternalExpenseService,
        ScheduledTransactionService, TemplateDetailService, TemplateSummaryService,
        TemplateTransactionService,
    },
    startup::AppState,
};

impl FromRef<AppState> for TemplateDetailService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            budgeter_config_repo: Box::new(PostgresBudgeterConfigRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            external_expense_repo: Box::new(PostgresExternalExpenseRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            category_service: Box::new(CategoryService {
                ynab_category_repo: Box::new(PostgresYnabCategoryRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                }),
                ynab_category_meta_repo: Box::new(RedisYnabCategoryMetaRepo {
                    redis_conn: state.redis_conn.clone(),
                }),
                expense_categorization_repo: Box::new(PostgresExpenseCategorizationRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                }),
                ynab_client: state.ynab_client.clone(),
            }),
            scheduled_transaction_service: Box::new(ScheduledTransactionService {
                ynab_scheduled_transaction_repo: Box::new(PostgresYnabScheduledTransactionRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                }),
                ynab_scheduled_transaction_meta_repo: Box::new(
                    RedisYnabScheduledTransactionMetaRepo {
                        redis_conn: state.redis_conn.clone(),
                    },
                ),
                ynab_client: state.ynab_client.clone(),
            }),
        }
    }
}

impl FromRef<AppState> for TemplateSummaryService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            budgeter_config_repo: Box::new(PostgresBudgeterConfigRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            external_expense_repo: Box::new(PostgresExternalExpenseRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            category_service: Box::new(CategoryService {
                ynab_category_repo: Box::new(PostgresYnabCategoryRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                }),
                ynab_category_meta_repo: Box::new(RedisYnabCategoryMetaRepo {
                    redis_conn: state.redis_conn.clone(),
                }),
                expense_categorization_repo: Box::new(PostgresExpenseCategorizationRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                }),
                ynab_client: state.ynab_client.clone(),
            }),
            scheduled_transaction_service: Box::new(ScheduledTransactionService {
                ynab_scheduled_transaction_repo: Box::new(PostgresYnabScheduledTransactionRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                }),
                ynab_scheduled_transaction_meta_repo: Box::new(
                    RedisYnabScheduledTransactionMetaRepo {
                        redis_conn: state.redis_conn.clone(),
                    },
                ),
                ynab_client: state.ynab_client.clone(),
            }),
        }
    }
}

impl FromRef<AppState> for TemplateTransactionService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            scheduled_transaction_service: Box::new(ScheduledTransactionService {
                ynab_scheduled_transaction_repo: Box::new(PostgresYnabScheduledTransactionRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                }),
                ynab_scheduled_transaction_meta_repo: Box::new(
                    RedisYnabScheduledTransactionMetaRepo {
                        redis_conn: state.redis_conn.clone(),
                    },
                ),
                ynab_client: state.ynab_client.clone(),
            }),
            ynab_category_repo: Box::new(PostgresYnabCategoryRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
            ynab_client: state.ynab_client.clone(),
        }
    }
}

impl FromRef<AppState> for BudgeterService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            budgeter_config_repo: Box::new(PostgresBudgeterConfigRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
        }
    }
}

impl FromRef<AppState> for ExternalExpenseService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            external_expense_repo: Box::new(PostgresExternalExpenseRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
        }
    }
}

impl FromRef<AppState> for ExpenseCategorizationService {
    fn from_ref(state: &AppState) -> Self {
        Self {
            expense_categorization_repo: Box::new(PostgresExpenseCategorizationRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            }),
        }
    }
}

pub fn get_budget_template_routes() -> Router<AppState> {
    Router::new()
        .route("/details", get(template_details))
        .route("/summary", get(template_summary))
        .route("/transactions", get(template_transactions))
        .route("/budgeters", get(get_all_budgeters))
        .route("/budgeter", post(create_budgeter))
        .route(
            "/budgeter/:budgeter_id",
            get(get_budgeter)
                .put(update_budgeter)
                .delete(delete_budgeter),
        )
        .route("/external_expenses", get(get_all_external_expenses))
        .route("/external_expense", post(create_external_expense))
        .route(
            "/external_expense/:external_expense_id",
            get(get_external_expense)
                .put(update_external_expense)
                .delete(delete_external_expense),
        )
        .route(
            "/expenses_categorization",
            get(get_all_expenses_categorization).put(update_all_expenses_categorization),
        )
        .route(
            "/expense_categorization/:expense_categorization_id",
            get(get_expense_categorization).put(update_expense_categorization),
        )
}
