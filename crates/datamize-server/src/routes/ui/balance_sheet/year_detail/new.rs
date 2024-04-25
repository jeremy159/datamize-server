use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum_extra::extract::Form;
use datamize_domain::SaveYear;
use serde_json::json;

use crate::{
    error::DatamizeResult,
    services::balance_sheet::{DynFinResService, DynMonthService, DynYearService},
};

#[derive(Template, Default)]
#[template(path = "partials/year-details/new.html")]
pub struct YearFormTemplate {
    year: Option<i32>,
    error: Option<String>,
}

pub async fn get() -> impl IntoResponse {
    YearFormTemplate::default()
}

pub async fn post(
    State((year_service, _, _)): State<(DynYearService, DynMonthService, DynFinResService)>,
    Form(payload): Form<SaveYear>,
) -> DatamizeResult<impl IntoResponse> {
    match year_service.create_year(payload.clone()).await {
        Ok(year) => {
            Ok([("Hx-Location", json!({"path": &format!("/balance_sheet/years/{}", year.year), "target": "#main", "swap": "outerHTML", "select": "#main"}).to_string())].into_response())
        }
        Err(e) => Ok(YearFormTemplate {
            year: Some(payload.year),
            error: Some(e.to_string()),
        }
        .into_response()),
    }
}
