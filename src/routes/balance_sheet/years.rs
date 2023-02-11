use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    domain::{NetTotal, NetTotalType, YearSummary},
    error::HttpJsonAppResult,
    startup::AppState,
};

/// Returns a summary of all the years with balance sheets.
pub async fn balance_sheet_years(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<YearSummary>> {
    let db_conn_pool = app_state.db_conn_pool;

    let db_rows = sqlx::query!(
        r#"
        SELECT
            y.id as year_id,
            y.year,
            n.id as net_total_id,
            n.type,
            n.total,
            n.percent_var,
            n.balance_var,
            n.year_id as net_total_year_id
        FROM balance_sheet_years AS y
        JOIN balance_sheet_net_totals_years AS n ON year_id = n.year_id;
        "#
    )
    .fetch_all(&db_conn_pool)
    .await?;

    println!("db_rows = {:?}", db_rows);
    // TODO: transform above query to YearSummary struct.

    let years: Vec<YearSummary> = vec![
        YearSummary {
            id: Uuid::new_v4(),
            year: 2021,
            net_totals: vec![
                NetTotal {
                    id: Uuid::new_v4(),
                    net_type: NetTotalType::Asset,
                    total: 161694000,
                    percent_var: 1.815,
                    balance_var: 104258000,
                },
                NetTotal {
                    id: Uuid::new_v4(),
                    net_type: NetTotalType::Portfolio,
                    total: 46895000,
                    percent_var: 0.904,
                    balance_var: 22260000,
                },
            ],
        },
        YearSummary {
            id: Uuid::new_v4(),
            year: 2022,
            net_totals: vec![
                NetTotal {
                    id: Uuid::new_v4(),
                    net_type: NetTotalType::Asset,
                    total: 222976000,
                    percent_var: 0.379,
                    balance_var: 61282000,
                },
                NetTotal {
                    id: Uuid::new_v4(),
                    net_type: NetTotalType::Portfolio,
                    total: 57762000,
                    percent_var: 0.232,
                    balance_var: 10867000,
                },
            ],
        },
    ];
    Ok(Json(years))
}
