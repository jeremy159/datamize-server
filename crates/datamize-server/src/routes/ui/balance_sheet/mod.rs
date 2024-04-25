mod edit_balance;
mod financial_resource;
mod year_detail;

use axum::{
    routing::{get, post},
    Router,
};
use db_postgres::{
    balance_sheet::{
        PostgresFinResRepo, PostgresMonthRepo, PostgresSavingRateRepo, PostgresYearRepo,
    },
    budget_providers::{
        external::PostgresExternalAccountRepo,
        ynab::{PostgresYnabAccountRepo, PostgresYnabTransactionRepo},
    },
};
use db_redis::budget_providers::{
    external::RedisEncryptionKeyRepo,
    ynab::{RedisYnabAccountMetaRepo, RedisYnabTransactionMetaRepo},
};

use crate::{
    services::{
        balance_sheet::{
            DynFinResService, DynMonthService, DynRefreshFinResService, DynSavingRateService,
            DynYearService, FinResService, MonthService, RefreshFinResService, SavingRateService,
            YearService,
        },
        budget_providers::{
            DynExternalAccountService, DynYnabAccountService, ExternalAccountService,
            TransactionService, YnabAccountService,
        },
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
        external_acount_service.clone(),
        app_state.ynab_client.clone(),
    );

    let ynab_account_repo = PostgresYnabAccountRepo::new_arced(app_state.db_conn_pool.clone());
    let ynab_account_meta_repo =
        RedisYnabAccountMetaRepo::new_arced(app_state.redis_conn_pool.clone());
    let ynab_account_service = YnabAccountService::new_arced(
        ynab_account_repo,
        ynab_account_meta_repo,
        app_state.ynab_client.clone(),
    );

    Router::new()
        .merge(get_year_routes(
            year_service,
            month_service,
            fin_res_service.clone(),
        ))
        .merge(get_fin_res_routes(
            fin_res_service,
            ynab_account_service,
            external_acount_service,
        ))
        .merge(get_refresh_fin_res_routes(refresh_fin_res_service))
}

fn get_year_routes<S>(
    year_service: DynYearService,
    month_service: DynMonthService,
    fin_res_service: DynFinResService,
) -> Router<S> {
    Router::new()
        .route(
            "/years/new",
            get(year_detail::new::get).post(year_detail::new::post),
        )
        .route(
            "/years/:year",
            get(year_detail::get).delete(year_detail::delete),
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
        .with_state((year_service, month_service, fin_res_service))
}

fn get_fin_res_routes<S: Clone + Send + Sync + 'static>(
    fin_res_service: DynFinResService,
    ynab_account_service: DynYnabAccountService,
    external_account_service: DynExternalAccountService,
) -> Router<S> {
    let first = Router::new()
        .route(
            "/years/:year/:month/:fin_res_id/edit_balance",
            get(edit_balance::get).put(edit_balance::put),
        )
        .with_state(fin_res_service.clone());

    let second = Router::new()
        .route(
            "/years/:year/:fin_res_id",
            get(financial_resource::get).delete(financial_resource::delete),
        )
        .route(
            "/years/:year/:fin_res_id/edit",
            get(financial_resource::edit::get).put(financial_resource::edit::put),
        )
        .route(
            "/years/:year/new",
            get(financial_resource::new::get).post(financial_resource::new::post),
        )
        .with_state((
            fin_res_service,
            ynab_account_service,
            external_account_service,
        ));

    Router::new().merge(first).merge(second)
}

fn get_refresh_fin_res_routes<S>(refresh_fin_res_service: DynRefreshFinResService) -> Router<S> {
    Router::new()
        .route(
            "/resources/refresh",
            post(financial_resource::refresh::post),
        )
        .with_state(refresh_fin_res_service)
}
