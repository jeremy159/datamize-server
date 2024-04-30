use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use datamize_domain::Month;

use crate::{
    error::DatamizeResult,
    routes::ui::{num_to_currency, num_to_currency_rounded, num_to_percentage_f32},
    services::balance_sheet::{DynMonthService, DynYearService},
};

pub async fn get(
    Path(year): Path<i32>,
    State((_, month_service)): State<(DynYearService, DynMonthService)>,
) -> DatamizeResult<impl IntoResponse> {
    let months = month_service.get_all_months_from_year(year).await?;

    Ok(YearDetailsTotalMonthlyTemplate { months })
}

#[derive(Template)]
#[template(path = "partials/year-details/total-monthly.html")]
struct YearDetailsTotalMonthlyTemplate {
    months: Vec<Month>,
}
