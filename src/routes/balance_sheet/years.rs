use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    domain::{TotalSummary, YearSummary},
    error::HttpJsonAppResult,
    startup::AppState,
};

/// Returns a summary of all the years with balance sheets.
pub async fn balance_sheet_years(
    State(_app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<YearSummary>> {
    let years: Vec<YearSummary> = vec![
        YearSummary {
            id: Uuid::new_v4(),
            year: 2021,
            net_assets: TotalSummary {
                total: 161694000,
                percent_variation: 1.815,
                balance_variation: 104258000,
            },
            net_portfolio: TotalSummary {
                total: 46895000,
                percent_variation: 0.904,
                balance_variation: 22260000,
            },
        },
        YearSummary {
            id: Uuid::new_v4(),
            year: 2022,
            net_assets: TotalSummary {
                total: 222976000,
                percent_variation: 0.379,
                balance_variation: 61282000,
            },
            net_portfolio: TotalSummary {
                total: 57762000,
                percent_variation: 0.232,
                balance_variation: 10867000,
            },
        },
    ];
    Ok(Json(years))
}
