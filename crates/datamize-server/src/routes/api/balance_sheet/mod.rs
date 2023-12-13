mod month;
mod months;
mod refresh_resources;
mod resource;
mod resources;
mod saving_rate;
mod saving_rates;
#[cfg(test)]
mod tests;
mod year;
mod years;

use axum::{
    routing::{get, post},
    Router,
};
use db_postgres::{
    balance_sheet::{
        PostgresFinResRepo, PostgresMonthRepo, PostgresSavingRateRepo, PostgresYearRepo,
    },
    budget_providers::{external::PostgresExternalAccountRepo, ynab::PostgresYnabTransactionRepo},
};
use db_redis::budget_providers::{
    external::RedisEncryptionKeyRepo, ynab::RedisYnabTransactionMetaRepo,
};
use month::*;
use months::*;
use refresh_resources::*;
use resource::*;
use resources::*;
use saving_rate::*;
use saving_rates::*;
use year::*;
use years::*;

use crate::{
    services::{
        balance_sheet::{
            DynFinResService, DynMonthService, DynRefreshFinResService, DynSavingRateService,
            DynYearService, FinResService, MonthService, RefreshFinResService, SavingRateService,
            YearService,
        },
        budget_providers::{ExternalAccountService, TransactionService},
    },
    startup::AppState,
};

pub fn get_balance_sheets_routes<S: Clone + Send + Sync + 'static>(
    app_state: &AppState,
) -> Router<S> {
    let year_repo = PostgresYearRepo::new_arced(app_state.db_conn_pool.clone());
    let month_repo = PostgresMonthRepo::new_arced(app_state.db_conn_pool.clone());
    let fin_res_repo = PostgresFinResRepo::new_arced(app_state.db_conn_pool.clone());
    let year_service = YearService::new_arced(year_repo.clone());
    let month_service = MonthService::new_arced(month_repo.clone());
    let fin_res_service =
        FinResService::new_arced(fin_res_repo.clone(), month_repo.clone(), year_repo.clone());
    let saving_rate_repo = PostgresSavingRateRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_transaction_repo =
        PostgresYnabTransactionRepo::new_boxed(app_state.db_conn_pool.clone());
    let ynab_transaction_meta_repo =
        RedisYnabTransactionMetaRepo::new_boxed(app_state.redis_conn.clone());
    let transaction_service = TransactionService::new_boxed(
        ynab_transaction_repo,
        ynab_transaction_meta_repo,
        app_state.ynab_client.clone(),
    );
    let saving_rate_service = SavingRateService::new_boxed(saving_rate_repo, transaction_service);
    let external_account_repo =
        PostgresExternalAccountRepo::new_boxed(app_state.db_conn_pool.clone());
    let encryption_key_repo = RedisEncryptionKeyRepo::new_boxed(app_state.redis_conn.clone());
    let external_acount_service =
        ExternalAccountService::new_boxed(external_account_repo, encryption_key_repo);
    let refresh_fin_res_service = RefreshFinResService::new_boxed(
        fin_res_repo,
        month_repo,
        year_repo,
        external_acount_service,
        app_state.ynab_client.clone(),
    );

    Router::new()
        .merge(get_year_routes(year_service))
        .merge(get_month_routes(month_service))
        .merge(get_fin_res_routes(fin_res_service))
        .merge(get_saving_rate_routes(saving_rate_service))
        .merge(get_refresh_fin_res_routes(refresh_fin_res_service))
}

fn get_year_routes<S>(year_service: DynYearService) -> Router<S> {
    Router::new()
        .route(
            "/years",
            get(balance_sheet_years).post(create_balance_sheet_year),
        )
        .route(
            "/years/:year",
            get(balance_sheet_year).delete(delete_balance_sheet_year),
        )
        .with_state(year_service)
}

fn get_month_routes<S>(month_service: DynMonthService) -> Router<S> {
    Router::new()
        .route("/months", get(all_balance_sheet_months))
        .route(
            "/years/:year/months",
            get(balance_sheet_months).post(create_balance_sheet_month),
        )
        .route(
            "/years/:year/months/:month",
            get(balance_sheet_month).delete(delete_balance_sheet_month),
        )
        .with_state(month_service)
}

fn get_fin_res_routes<S>(fin_res_service: DynFinResService) -> Router<S> {
    Router::new()
        .route(
            "/resources",
            get(all_balance_sheet_resources).post(create_balance_sheet_resource),
        )
        .route(
            "/resources/:resource_id",
            get(balance_sheet_resource)
                .put(update_balance_sheet_resource)
                .delete(delete_balance_sheet_resource),
        )
        .route("/years/:year/resources", get(balance_sheet_resources))
        .with_state(fin_res_service)
}

fn get_saving_rate_routes<S>(saving_rate_service: DynSavingRateService) -> Router<S> {
    Router::new()
        .route("/saving_rates", post(create_balance_sheet_saving_rate))
        .route(
            "/saving_rates/:saving_rate_id",
            get(balance_sheet_saving_rate)
                .put(update_balance_sheet_saving_rate)
                .delete(delete_balance_sheet_saving_rate),
        )
        .route("/years/:year/saving_rates", get(balance_sheet_saving_rates))
        .with_state(saving_rate_service)
}

fn get_refresh_fin_res_routes<S>(refresh_fin_res_service: DynRefreshFinResService) -> Router<S> {
    Router::new()
        .route("/resources/refresh", post(refresh_balance_sheet_resources))
        .with_state(refresh_fin_res_service)
}
