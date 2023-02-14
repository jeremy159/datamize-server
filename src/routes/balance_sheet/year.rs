use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    common::{build_months, get_year},
    db,
    domain::{UpdateYear, YearDetail},
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
};

/// Returns a detailed year with its balance sheet and its saving rates.
pub async fn balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(get_year(&db_conn_pool, year).await?))
}

// TODO: To recompute net totals of current year after update.
/// Updates the saving rates of the received year.
pub async fn update_balance_sheet_year(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    Json(body): Json<UpdateYear>,
) -> HttpJsonAppResult<YearDetail> {
    let db_conn_pool = app_state.db_conn_pool;

    let Some(year_data) = db::get_year_data(&db_conn_pool, year)
    .await? else {
        return Err(AppError::ResourceNotFound);
    };

    let net_totals = db::get_year_net_totals_for(&db_conn_pool, year_data.id).await?;

    db::update_saving_rates(&db_conn_pool, &body.saving_rates).await?;

    let months = build_months(&db_conn_pool, year_data.id).await?;

    Ok(Json(YearDetail {
        id: year_data.id,
        year: year_data.year,
        net_totals,
        saving_rates: body.saving_rates,
        months,
    }))
}
