use std::{cmp::Ordering, collections::HashMap};

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{Month, MonthNum, NetTotal, NetTotalType};

#[derive(sqlx::FromRow, Debug)]
pub struct MonthData {
    pub id: Uuid,
    pub month: i16,
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_month_data(
    db_conn_pool: &PgPool,
    month: MonthNum,
    year: i32,
) -> Result<Option<MonthData>, sqlx::Error> {
    sqlx::query_as!(
        MonthData,
        r#"
        SELECT
            m.id,
            month
        FROM balance_sheet_months AS m
        JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1
        WHERE m.month = $2;
        "#,
        year,
        month as i16,
    )
    .fetch_optional(db_conn_pool)
    .await
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_months(db_conn_pool: &PgPool, year: i32) -> Result<Vec<Month>, sqlx::Error> {
    let mut months = HashMap::<Uuid, Month>::new();

    let db_rows = sqlx::query!(
        r#"
        SELECT
            m.id as month_id,
            m.month,
            n.id as net_total_id,
            n.type,
            n.total,
            n.percent_var,
            n.balance_var,
            n.month_id as net_total_month_id
        FROM balance_sheet_months AS m
        JOIN balance_sheet_net_totals_months AS n ON month_id = n.month_id
        JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1;
        "#,
        year,
    )
    .fetch_all(db_conn_pool)
    .await?;

    for r in db_rows
        .into_iter()
        .filter(|v| v.month_id == v.net_total_month_id)
    {
        let is_net_assets_total = r.r#type == NetTotalType::Asset.to_string();
        let net_assets = match is_net_assets_total {
            true => NetTotal {
                id: r.net_total_id,
                net_type: r.r#type.parse().unwrap(),
                total: r.total,
                percent_var: r.percent_var,
                balance_var: r.balance_var,
            },
            false => NetTotal::new_asset(),
        };

        let net_portfolio = match r.r#type == NetTotalType::Portfolio.to_string() {
            true => NetTotal {
                id: r.net_total_id,
                net_type: r.r#type.parse().unwrap(),
                total: r.total,
                percent_var: r.percent_var,
                balance_var: r.balance_var,
            },
            false => NetTotal::new_portfolio(),
        };

        months
            .entry(r.month_id)
            .and_modify(|y| {
                if is_net_assets_total {
                    y.net_assets = net_assets.clone();
                } else {
                    y.net_portfolio = net_portfolio.clone();
                }
            })
            .or_insert_with(|| Month {
                id: r.month_id,
                month: r.month.try_into().unwrap(),
                year,
                net_assets,
                net_portfolio,
            });
    }

    let mut months = months.into_values().collect::<Vec<_>>();

    months.sort_by(|a, b| a.month.cmp(&b.month));

    Ok(months)
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_all_months(db_conn_pool: &PgPool) -> Result<Vec<Month>, sqlx::Error> {
    let mut months = HashMap::<Uuid, Month>::new();

    let db_rows = sqlx::query!(
        r#"
        SELECT
            m.id as month_id,
            m.month,
            n.id as net_total_id,
            n.type,
            n.total,
            n.percent_var,
            n.balance_var,
            n.month_id as net_total_month_id,
            y.year
        FROM balance_sheet_months AS m
        JOIN balance_sheet_net_totals_months AS n ON month_id = n.month_id
        JOIN balance_sheet_years AS y ON y.id = m.year_id;
        "#
    )
    .fetch_all(db_conn_pool)
    .await?;

    for r in db_rows
        .into_iter()
        .filter(|v| v.month_id == v.net_total_month_id)
    {
        let is_net_assets_total = r.r#type == NetTotalType::Asset.to_string();
        let net_assets = match is_net_assets_total {
            true => NetTotal {
                id: r.net_total_id,
                net_type: r.r#type.parse().unwrap(),
                total: r.total,
                percent_var: r.percent_var,
                balance_var: r.balance_var,
            },
            false => NetTotal::new_asset(),
        };

        let net_portfolio = match r.r#type == NetTotalType::Portfolio.to_string() {
            true => NetTotal {
                id: r.net_total_id,
                net_type: r.r#type.parse().unwrap(),
                total: r.total,
                percent_var: r.percent_var,
                balance_var: r.balance_var,
            },
            false => NetTotal::new_portfolio(),
        };

        months
            .entry(r.month_id)
            .and_modify(|y| {
                if is_net_assets_total {
                    y.net_assets = net_assets.clone();
                } else {
                    y.net_portfolio = net_portfolio.clone();
                }
            })
            .or_insert_with(|| Month {
                id: r.month_id,
                month: r.month.try_into().unwrap(),
                year: r.year,
                net_assets,
                net_portfolio,
            });
    }

    let mut months = months.into_values().collect::<Vec<_>>();

    months.sort_by(|a, b| match a.year.cmp(&b.year) {
        Ordering::Equal => a.month.cmp(&b.month),
        other => other,
    });

    Ok(months)
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_month(
    db_conn_pool: &PgPool,
    month_num: MonthNum,
    year: i32,
) -> Result<Month, sqlx::Error> {
    let db_rows = sqlx::query!(
        r#"
        SELECT
            m.id as month_id,
            m.month,
            n.id as net_total_id,
            n.type,
            n.total,
            n.percent_var,
            n.balance_var,
            n.month_id as net_total_month_id
        FROM balance_sheet_months AS m
        JOIN balance_sheet_net_totals_months AS n ON month_id = n.month_id
        JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1
        WHERE m.month = $2;
        "#,
        year,
        month_num as i16,
    )
    .fetch_all(db_conn_pool)
    .await?;

    let mut month: Option<Month> = None;

    for r in db_rows {
        let is_net_assets_total = r.r#type == NetTotalType::Asset.to_string();

        match month {
            Some(ref mut m) => {
                if is_net_assets_total && m.net_assets.total != r.total {
                    m.net_assets = NetTotal {
                        id: r.net_total_id,
                        net_type: r.r#type.parse().unwrap(),
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                    };
                } else if !is_net_assets_total && m.net_portfolio.total != r.total {
                    m.net_portfolio = NetTotal {
                        id: r.net_total_id,
                        net_type: r.r#type.parse().unwrap(),
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                    };
                }
            }
            None => {
                let net_assets = match is_net_assets_total {
                    true => NetTotal {
                        id: r.net_total_id,
                        net_type: r.r#type.parse().unwrap(),
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                    },
                    false => NetTotal::new_asset(),
                };

                let net_portfolio = match r.r#type == NetTotalType::Portfolio.to_string() {
                    true => NetTotal {
                        id: r.net_total_id,
                        net_type: r.r#type.parse().unwrap(),
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                    },
                    false => NetTotal::new_portfolio(),
                };

                month = Some(Month {
                    id: r.month_id,
                    month: r.month.try_into().unwrap(),
                    year,
                    net_assets,
                    net_portfolio,
                });
            }
        }
    }

    month.ok_or(sqlx::Error::RowNotFound)
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

    insert_monthly_net_totals(
        db_conn_pool,
        month.id,
        [&month.net_assets, &month.net_portfolio],
    )
    .await?;

    Ok(())
}

pub async fn insert_monthly_net_totals(
    db_conn_pool: &PgPool,
    month_id: Uuid,
    net_totals: [&NetTotal; 2],
) -> Result<(), sqlx::Error> {
    for nt in net_totals {
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_months (id, type, total, percent_var, balance_var, month_id)
            VALUES ($1, $2, $3, $4, $5, $6)
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
            month_id,
        )
        .execute(db_conn_pool)
        .await?;
    }

    Ok(())
}

#[tracing::instrument(skip(db_conn_pool))]
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

#[tracing::instrument(skip(db_conn_pool))]
pub async fn get_months_of_resource(
    db_conn_pool: &PgPool,
    resource_id: Uuid,
) -> Result<Vec<MonthData>, sqlx::Error> {
    let mut months: Vec<MonthData> = vec![];

    let db_rows = sqlx::query!(
        r#"
            SELECT
                m.*
            FROM balance_sheet_months AS m
            JOIN balance_sheet_resources_months AS rm ON m.id = rm.month_id AND rm.resource_id = $1
            ORDER BY m.month ASC;
            "#,
        resource_id,
    )
    .fetch_all(db_conn_pool)
    .await?;

    for r in db_rows {
        months.push(MonthData {
            id: r.id,
            month: r.month,
        })
    }

    Ok(months)
}

#[tracing::instrument(skip(db_conn_pool))]
pub async fn delete_month(
    db_conn_pool: &PgPool,
    month_num: MonthNum,
    year: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            DELETE FROM balance_sheet_months
            WHERE month = $1 AND year_id in (SELECT y.id
            FROM balance_sheet_years AS y WHERE y.year = $2);
        "#,
        month_num as i16,
        year,
    )
    .execute(db_conn_pool)
    .await?;

    Ok(())
}
