use askama_axum::IntoResponse;
use axum::extract::State;
use chrono::{Datelike, Local};

use crate::{
    error::DatamizeResult,
    services::balance_sheet::{DynMonthService, DynYearService},
};

use super::YearDetailsTemplate;

pub async fn get(
    State((year_service, month_service)): State<(DynYearService, DynMonthService)>,
) -> DatamizeResult<impl IntoResponse> {
    let current_year = Local::now().date_naive().year();
    let years = year_service
        .get_all_years_num()
        .await
        .unwrap_or(vec![current_year]);

    let year = if years.contains(&current_year) {
        current_year
    } else {
        *years.last().unwrap_or(&current_year)
    };

    YearDetailsTemplate::build(year_service, month_service, year).await
}
