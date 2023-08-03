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
    get_redis_conn,
    services::budget_template::{
        BudgeterService, CategoryService, ExpenseCategorizationService, ExternalExpenseService,
        ScheduledTransactionService, TemplateDetailService, TemplateSummaryService,
        TemplateTransactionService,
    },
    startup::AppState,
};

impl FromRef<AppState>
    for TemplateDetailService<PostgresBudgeterConfigRepo, PostgresExternalExpenseRepo>
{
    fn from_ref(state: &AppState) -> Self {
        let redis_conn_1 = get_redis_conn(&state.redis_conn_pool)
            .expect("failed to get redis connection from pool");
        let redis_conn_2 = get_redis_conn(&state.redis_conn_pool)
            .expect("failed to get redis connection from pool");

        Self {
            budgeter_config_repo: PostgresBudgeterConfigRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
            external_expense_repo: PostgresExternalExpenseRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
            category_service: Box::new(CategoryService {
                ynab_category_repo: PostgresYnabCategoryRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                },
                ynab_category_meta_repo: RedisYnabCategoryMetaRepo {
                    redis_conn: redis_conn_1,
                },
                expense_categorization_repo: PostgresExpenseCategorizationRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                },
                ynab_client: state.ynab_client.clone(),
            }),
            scheduled_transaction_service: Box::new(ScheduledTransactionService {
                ynab_scheduled_transaction_repo: PostgresYnabScheduledTransactionRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                },
                ynab_scheduled_transaction_meta_repo: RedisYnabScheduledTransactionMetaRepo {
                    redis_conn: redis_conn_2,
                },
                ynab_client: state.ynab_client.clone(),
            }),
            ynab_client: state.ynab_client.clone(),
        }
    }
}

impl FromRef<AppState>
    for TemplateSummaryService<PostgresBudgeterConfigRepo, PostgresExternalExpenseRepo>
{
    fn from_ref(state: &AppState) -> Self {
        let redis_conn_1 = get_redis_conn(&state.redis_conn_pool)
            .expect("failed to get redis connection from pool");
        let redis_conn_2 = get_redis_conn(&state.redis_conn_pool)
            .expect("failed to get redis connection from pool");

        Self {
            budgeter_config_repo: PostgresBudgeterConfigRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
            external_expense_repo: PostgresExternalExpenseRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
            category_service: Box::new(CategoryService {
                ynab_category_repo: PostgresYnabCategoryRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                },
                ynab_category_meta_repo: RedisYnabCategoryMetaRepo {
                    redis_conn: redis_conn_1,
                },
                expense_categorization_repo: PostgresExpenseCategorizationRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                },
                ynab_client: state.ynab_client.clone(),
            }),
            scheduled_transaction_service: Box::new(ScheduledTransactionService {
                ynab_scheduled_transaction_repo: PostgresYnabScheduledTransactionRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                },
                ynab_scheduled_transaction_meta_repo: RedisYnabScheduledTransactionMetaRepo {
                    redis_conn: redis_conn_2,
                },
                ynab_client: state.ynab_client.clone(),
            }),
            ynab_client: state.ynab_client.clone(),
        }
    }
}

impl FromRef<AppState> for TemplateTransactionService<PostgresYnabCategoryRepo> {
    fn from_ref(state: &AppState) -> Self {
        let redis_conn = get_redis_conn(&state.redis_conn_pool)
            .expect("failed to get redis connection from pool");

        Self {
            scheduled_transaction_service: Box::new(ScheduledTransactionService {
                ynab_scheduled_transaction_repo: PostgresYnabScheduledTransactionRepo {
                    db_conn_pool: state.db_conn_pool.clone(),
                },
                ynab_scheduled_transaction_meta_repo: RedisYnabScheduledTransactionMetaRepo {
                    redis_conn,
                },
                ynab_client: state.ynab_client.clone(),
            }),
            ynab_category_repo: PostgresYnabCategoryRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
            ynab_client: state.ynab_client.clone(),
        }
    }
}

impl FromRef<AppState> for BudgeterService<PostgresBudgeterConfigRepo> {
    fn from_ref(state: &AppState) -> Self {
        Self {
            budgeter_config_repo: PostgresBudgeterConfigRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
        }
    }
}

impl FromRef<AppState> for ExternalExpenseService<PostgresExternalExpenseRepo> {
    fn from_ref(state: &AppState) -> Self {
        Self {
            external_expense_repo: PostgresExternalExpenseRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
        }
    }
}

impl FromRef<AppState> for ExpenseCategorizationService<PostgresExpenseCategorizationRepo> {
    fn from_ref(state: &AppState) -> Self {
        Self {
            expense_categorization_repo: PostgresExpenseCategorizationRepo {
                db_conn_pool: state.db_conn_pool.clone(),
            },
        }
    }
}

pub fn get_budget_template_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/details",
            get(template_details::<
                TemplateDetailService<PostgresBudgeterConfigRepo, PostgresExternalExpenseRepo>,
            >),
        )
        .route(
            "/summary",
            get(template_summary::<
                TemplateSummaryService<PostgresBudgeterConfigRepo, PostgresExternalExpenseRepo>,
            >),
        )
        .route(
            "/transactions",
            get(template_transactions::<TemplateTransactionService<PostgresYnabCategoryRepo>>),
        )
        .route(
            "/budgeters",
            get(get_all_budgeters::<BudgeterService<PostgresBudgeterConfigRepo>>),
        )
        .route(
            "/budgeter",
            post(create_budgeter::<BudgeterService<PostgresBudgeterConfigRepo>>),
        )
        .route(
            "/budgeter/:budgeter_id",
            get(get_budgeter::<BudgeterService<PostgresBudgeterConfigRepo>>)
                .put(update_budgeter::<BudgeterService<PostgresBudgeterConfigRepo>>)
                .delete(delete_budgeter::<BudgeterService<PostgresBudgeterConfigRepo>>),
        )
        .route(
            "/external_expenses",
            get(get_all_external_expenses::<ExternalExpenseService<PostgresExternalExpenseRepo>>),
        )
        .route(
            "/external_expense",
            post(create_external_expense::<ExternalExpenseService<PostgresExternalExpenseRepo>>),
        )
        .route(
            "/external_expense/:external_expense_id",
            get(get_external_expense::<ExternalExpenseService<PostgresExternalExpenseRepo>>)
                .put(update_external_expense::<ExternalExpenseService<PostgresExternalExpenseRepo>>)
                .delete(
                    delete_external_expense::<ExternalExpenseService<PostgresExternalExpenseRepo>>,
                ),
        )
        .route(
            "/expenses_categorization",
            get(get_all_expenses_categorization::<
                ExpenseCategorizationService<PostgresExpenseCategorizationRepo>,
            >)
            .put(
                update_all_expenses_categorization::<
                    ExpenseCategorizationService<PostgresExpenseCategorizationRepo>,
                >,
            ),
        )
        .route(
            "/expense_categorization/:expense_categorization_id",
            get(get_expense_categorization::<
                ExpenseCategorizationService<PostgresExpenseCategorizationRepo>,
            >)
            .put(
                update_expense_categorization::<
                    ExpenseCategorizationService<PostgresExpenseCategorizationRepo>,
                >,
            ),
        )
}
