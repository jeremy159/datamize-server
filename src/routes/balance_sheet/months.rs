use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    domain::{Month, SaveMonth},
    error::HttpJsonAppResult,
    startup::AppState,
};

use super::build_months;

pub async fn balance_sheet_months(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
) -> HttpJsonAppResult<Vec<Month>> {
    let db_conn_pool = app_state.db_conn_pool;

    #[derive(sqlx::FromRow, Debug)]
    struct YearData {
        id: Uuid,
    }

    let Some(year_data) = sqlx::query_as!(
        YearData,
        r#"
        SELECT id
        FROM balance_sheet_years
        WHERE year = $1;
        "#,
        year
    )
    .fetch_optional(&db_conn_pool)
    .await? else {
        return Err(crate::error::AppError::ResourceNotFound);
    };

    Ok(Json(build_months(&db_conn_pool, year_data.id).await?))
}

// TODO: When creating month, update net totals for this month compared to previous one if any.
pub async fn create_balance_sheet_month(
    Path(year): Path<i32>,
    State(app_state): State<AppState>,
    Json(body): Json<SaveMonth>,
) -> HttpJsonAppResult<Month> {
    let db_conn_pool = app_state.db_conn_pool;

    #[derive(sqlx::FromRow, Debug)]
    struct YearData {
        id: Uuid,
    }

    let Some(year_data) = sqlx::query_as!(
        YearData,
        r#"
        SELECT id
        FROM balance_sheet_years
        WHERE year = $1;
        "#,
        year
    )
    .fetch_optional(&db_conn_pool)
    .await? else {
        return Err(crate::error::AppError::ResourceNotFound);
    };

    let None = sqlx::query!(
        r#"
        SELECT *
        FROM balance_sheet_months
        WHERE year_id = $1 AND month = $2;
        "#,
        year_data.id,
        body.month as i16,
    )
    .fetch_optional(&db_conn_pool)
    .await? else {
        return Err(crate::error::AppError::MonthAlreadyExist);
    };

    let month = Month::new(body.month);

    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_months (id, month, year_id)
        VALUES ($1, $2, $3);
        "#,
        month.id,
        month.month as i16,
        year_data.id,
    )
    .execute(&db_conn_pool)
    .await?;

    Ok(Json(month))
}
