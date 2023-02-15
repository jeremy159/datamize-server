use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    FinancialResource, Month, NetTotal, NetTotalType, ResourceCategory, ResourceType,
};

#[derive(sqlx::FromRow, Debug)]
pub struct MonthData {
    pub id: Uuid,
    pub month: i16,
}

#[tracing::instrument(skip_all)]
pub async fn get_month_data(
    db_conn_pool: &PgPool,
    year_id: Uuid,
    month: i16,
) -> Result<Option<MonthData>, sqlx::Error> {
    sqlx::query_as!(
        MonthData,
        r#"
        SELECT
            id,
            month
        FROM balance_sheet_months
        WHERE year_id = $1 AND month = $2;
        "#,
        year_id,
        month,
    )
    .fetch_optional(db_conn_pool)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn get_months_data(
    db_conn_pool: &PgPool,
    year_id: Uuid,
) -> Result<Vec<MonthData>, sqlx::Error> {
    sqlx::query_as!(
        MonthData,
        r#"
        SELECT
            id,
            month
        FROM balance_sheet_months
        WHERE year_id = $1;
        "#,
        year_id,
    )
    .fetch_all(db_conn_pool)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn add_new_month(
    db_conn_pool: &PgPool,
    month: &Month,
    year_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO balance_sheet_months (id, month, year_id)
        VALUES ($1, $2, $3);
        "#,
        month.id,
        month.month as i16,
        year_id,
    )
    .execute(db_conn_pool)
    .await?;

    for nt in &month.net_totals {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_months (id, type, total, percent_var, balance_var, month_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            nt.id,
            nt.net_type.to_string(),
            nt.total,
            nt.percent_var,
            nt.balance_var,
            month.id,
        )
        .execute(db_conn_pool)
        .await?;
    }

    for fr in &month.resources {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, category, type, balance, editable, month_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            category = EXCLUDED.category,
            type = EXCLUDED.type,
            balance = EXCLUDED.balance,
            editable = EXCLUDED.editable;
            "#,
            fr.id,
            fr.name,
            fr.category.to_string(),
            fr.resource_type.to_string(),
            fr.balance,
            fr.editable,
            month.id
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn get_month_net_totals_for(
    db_conn_pool: &PgPool,
    month_id: Uuid,
) -> Result<Vec<NetTotal>, sqlx::Error> {
    sqlx::query_as!(
        NetTotal,
        r#"
        SELECT
            id,
            type AS "net_type: NetTotalType",
            total,
            percent_var,
            balance_var
        FROM balance_sheet_net_totals_months
        WHERE month_id = $1;
        "#,
        month_id,
    )
    .fetch_all(db_conn_pool)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn update_month_net_totals(
    db_conn_pool: &PgPool,
    net_totals: &[NetTotal],
) -> Result<(), sqlx::Error> {
    for nt in net_totals {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_months (id, type, total, percent_var, balance_var)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET type = EXCLUDED.type,
            total = EXCLUDED.total,
            percent_var = EXCLUDED.percent_var,
            balance_var = EXCLUDED.balance_var;
            "#,
            nt.id,
            nt.net_type.to_string(),
            nt.total,
            nt.percent_var,
            nt.balance_var,
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn get_financial_resources_for(
    db_conn_pool: &PgPool,
    month_id: Uuid,
) -> Result<Vec<FinancialResource>, sqlx::Error> {
    sqlx::query_as!(
        FinancialResource,
        r#"
            SELECT
                id,
                name,
                category AS "category: ResourceCategory",
                type AS "resource_type: ResourceType",
                balance,
                editable
            FROM balance_sheet_resources
            WHERE month_id = $1;
            "#,
        month_id,
    )
    .fetch_all(db_conn_pool)
    .await
}

#[tracing::instrument(skip_all)]
pub async fn update_financial_resources(
    db_conn_pool: &PgPool,
    resources: &[FinancialResource],
) -> Result<(), sqlx::Error> {
    for f in resources {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, category, type, balance, editable)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            category = EXCLUDED.category,
            type = EXCLUDED.type,
            balance = EXCLUDED.balance,
            editable = EXCLUDED.editable;
            "#,
            f.id,
            f.name,
            f.category.to_string(),
            f.resource_type.to_string(),
            f.balance,
            f.editable,
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}
