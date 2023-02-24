use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    db,
    domain::{NetTotalType, SaveYear, YearDetail, YearSummary},
    error::{AppError, HttpJsonAppResult, JsonError},
    startup::AppState,
};

/// Returns a summary of all the years with balance sheets.
#[tracing::instrument(name = "Get a summary of all years", skip_all)]
pub async fn balance_sheet_years(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<YearSummary>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(db::get_years_summary(&db_conn_pool).await?))
}

/// Creates a new year if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this year compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_year(
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveYear>, JsonError>,
) -> impl IntoResponse {
    let db_conn_pool = app_state.db_conn_pool;

    let None = db::get_year_data(&db_conn_pool, body.year)
    .await? else {
        return Err(AppError::YearAlreadyExist);
    };

    let mut year = YearDetail::new(body.year);

    if let Ok(Some(prev_year)) = db::get_year_data(&db_conn_pool, year.year - 1).await {
        if let Ok(prev_net_totals) = db::get_year_net_totals_for(&db_conn_pool, prev_year.id).await
        {
            if let Some(prev_net_assets) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Asset)
            {
                year.update_net_assets_with_previous(prev_net_assets);
            }
            if let Some(prev_net_portfolio) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Portfolio)
            {
                year.update_net_portfolio_with_previous(prev_net_portfolio);
            }
        }
    }

    db::add_new_year(&db_conn_pool, &year).await?;

    Ok((StatusCode::CREATED, Json(year)))
}
