pub mod latest;
pub mod new;
pub mod resources;
pub mod total_monthly;

use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Path, State};
use datamize_domain::{BalancePerYearPerMonth, Month, YearlyBalances};
use serde_json::json;

use crate::{
    error::DatamizeResult,
    services::balance_sheet::{DynMonthService, DynYearService},
};

use self::new::YearFormTemplate;

pub async fn get(
    Path(year): Path<i32>,
    State((year_service, month_service)): State<(DynYearService, DynMonthService)>,
) -> DatamizeResult<impl IntoResponse> {
    YearDetailsTemplate::build(year_service, month_service, year).await
}

pub async fn delete(
    Path(year): Path<i32>,
    State((year_service, _)): State<(DynYearService, DynMonthService)>,
) -> DatamizeResult<impl IntoResponse> {
    _ = year_service.delete_year(year).await;

    Ok(match year_service.get_year(year - 1).await {
        Ok(prev_year) => [("Hx-Location", json!({"path": &format!("/balance_sheet/years/{}", prev_year.year), "target": "#main", "swap": "outerHTML", "select": "#main"}).to_string())].into_response(),
        Err(_) => [("Hx-Location", json!({"path": "/balance_sheet/years", "target": "#main", "swap": "outerHTML", "select": "#main"}).to_string())].into_response(),
    })
}

#[derive(Template)]
#[template(path = "pages/year-details.html")]
pub struct YearDetailsTemplate {
    new_year_form: YearFormTemplate,
    year: i32,
    years: Vec<i32>,
    months: Vec<Month>,
}

impl YearDetailsTemplate {
    pub async fn build(
        year_service: DynYearService,
        month_service: DynMonthService,
        year: i32,
    ) -> DatamizeResult<Self> {
        let years = year_service.get_all_years_num().await?;
        let months = month_service.get_all_months_from_year(year).await?;

        Ok(Self {
            new_year_form: YearFormTemplate::default(),
            year,
            years,
            months,
        })
    }
}

#[derive(Debug, Clone, Default)]
struct TotalRow {
    balances: BalancePerYearPerMonth,
}

impl YearlyBalances for TotalRow {
    fn balances(&self) -> &BalancePerYearPerMonth {
        &self.balances
    }

    fn balances_mut(&mut self) -> &mut BalancePerYearPerMonth {
        &mut self.balances
    }
}
