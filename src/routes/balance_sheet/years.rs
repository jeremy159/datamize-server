use std::collections::HashMap;

use axum::{extract::State, Json};
use uuid::Uuid;

use crate::{
    domain::{NetTotal, SaveYear, YearSummary},
    error::HttpJsonAppResult,
    startup::AppState,
};

/// Returns a summary of all the years with balance sheets.
pub async fn balance_sheet_years(
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<YearSummary>> {
    let db_conn_pool = app_state.db_conn_pool;

    let mut years = HashMap::<Uuid, YearSummary>::new();

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

    for r in db_rows
        .into_iter()
        .filter(|v| v.year_id == v.net_total_year_id)
    {
        let net_total = NetTotal {
            id: r.net_total_id,
            net_type: r.r#type.parse().unwrap(),
            total: r.total,
            percent_var: r.percent_var,
            balance_var: r.balance_var,
        };

        years
            .entry(r.year_id)
            .and_modify(|y| {
                y.net_totals.push(net_total.clone());
            })
            .or_insert_with(|| YearSummary {
                id: r.year_id,
                year: r.year,
                net_totals: vec![net_total],
            });
    }

    let mut years = years.into_values().collect::<Vec<_>>();

    years.sort_by(|a, b| a.year.cmp(&b.year));

    Ok(Json(years))
}

// TODO: When creating year, update net totals for this year compared to previous one if any.
/// Creates a new year if it doesn't already exist and returns the newly created entity.
pub async fn create_balance_sheet_year(
    State(app_state): State<AppState>,
    Json(body): Json<SaveYear>,
) -> HttpJsonAppResult<YearSummary> {
    let db_conn_pool = app_state.db_conn_pool;

    let None = sqlx::query!(
        r#"
        SELECT *
        FROM balance_sheet_years
        WHERE year = $1;
        "#,
        body.year,
    )
    .fetch_optional(&db_conn_pool)
    .await? else {
        return Err(crate::error::AppError::YearAlreadyExist);
    };

    let year = YearSummary::new(body.year);

    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_years (id, year)
        VALUES ($1, $2);
        "#,
        year.id,
        year.year,
    )
    .execute(&db_conn_pool)
    .await?;

    Ok(Json(year))
}
