use axum::{extract::State, Json};

use crate::{
    db,
    domain::{SaveYear, YearSummary},
    error::{AppError, HttpJsonAppResult},
    startup::AppState,
};

/// Returns a summary of all the years with balance sheets.
pub async fn balance_sheet_years(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<YearSummary>> {
    let db_conn_pool = app_state.db_conn_pool;

    Ok(Json(db::get_years_summary(&db_conn_pool).await?))
}

// TODO: When creating year, update net totals for this year compared to previous one if any.
/// Creates a new year if it doesn't already exist and returns the newly created entity.
pub async fn create_balance_sheet_year(
    State(app_state): State<AppState>,
    Json(body): Json<SaveYear>,
) -> HttpJsonAppResult<YearSummary> {
    let db_conn_pool = app_state.db_conn_pool;

    let None = db::get_year_data(&db_conn_pool, body.year)
    .await? else {
        return Err(AppError::YearAlreadyExist);
    };

    let year = YearSummary::new(body.year);

    db::add_new_year(&db_conn_pool, &year).await?;

    Ok(Json(year))
}
