use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::WithRejection;

use crate::{
    db::balance_sheet::{add_new_year, get_year_data, get_years_summary},
    error::{AppError, HttpJsonAppResult, JsonError},
    models::balance_sheet::{SaveYear, YearDetail, YearSummary},
    startup::AppState,
};

use super::common::{get_year, update_year_net_totals};

/// Returns a summary of all the years with balance sheets.
#[tracing::instrument(name = "Get a summary of all years", skip_all)]
pub async fn balance_sheet_years(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<YearSummary>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(get_years_summary(&db_conn_pool).await?))
}

/// Creates a new year if it doesn't already exist and returns the newly created entity.
/// Will also update net totals for this year compared to previous one if any.
#[tracing::instrument(skip_all)]
pub async fn create_balance_sheet_year(
    State(app_state): State<AppState>,
    WithRejection(Json(body), _): WithRejection<Json<SaveYear>, JsonError>,
) -> impl IntoResponse {
    let db_conn_pool = app_state.db_conn_pool;

    let Err(sqlx::Error::RowNotFound) =
        get_year_data(&db_conn_pool, body.year).await else
    {
        return Err(AppError::YearAlreadyExist);
    };

    let year = YearDetail::new(body.year);
    add_new_year(&db_conn_pool, &year).await?;

    update_year_net_totals(&db_conn_pool, body.year).await?;

    Ok((
        StatusCode::CREATED,
        Json(get_year(&db_conn_pool, year.year).await?),
    ))
}
