mod edit_balance;
mod year_detail;

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
        PostgresYnabTransactionRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_transaction_meta_repo =
        RedisYnabTransactionMetaRepo::new_arced(app_state.redis_conn_pool.clone());
    let transaction_service = TransactionService::new_arced(
        ynab_transaction_repo,
        ynab_transaction_meta_repo,
        app_state.ynab_client.clone(),
    );
    let saving_rate_service = SavingRateService::new_arced(saving_rate_repo, transaction_service);
    let external_account_repo =
        PostgresExternalAccountRepo::new_arced(app_state.db_conn_pool.clone());
    let encryption_key_repo = RedisEncryptionKeyRepo::new_arced(app_state.redis_conn_pool.clone());
    let external_acount_service =
        ExternalAccountService::new_arced(external_account_repo, encryption_key_repo);
    let refresh_fin_res_service = RefreshFinResService::new_arced(
        fin_res_repo,
        month_repo,
        year_repo,
        external_acount_service,
        app_state.ynab_client.clone(),
    );

    Router::new().merge(get_year_routes(month_service, fin_res_service))
}

fn get_year_routes<S>(
    month_service: DynMonthService,
    fin_res_service: DynFinResService,
) -> Router<S> {
    Router::new()
        .route("/years/:year", get(year_detail::get))
        .route(
            "/years/:year/:month/:fin_res_id/edit_balance",
            get(edit_balance::get).put(edit_balance::put),
        )
        .route(
            "/years/:year/total_monthly",
            get(year_detail::total_monthly::get),
        )
        .route(
            "/years/:year/total_assets",
            get(year_detail::total_assets::get),
        )
        .route(
            "/years/:year/total_liabilities",
            get(year_detail::total_liabilities::get),
        )
        .with_state((month_service, fin_res_service))
}

// fn get_refresh_fin_res_routes<S>(refresh_fin_res_service: DynRefreshFinResService) -> Router<S> {
//     Router::new()
//         .route("/resources/refresh", post(refresh_balance_sheet_resources))
//         .with_state(refresh_fin_res_service)
// }
