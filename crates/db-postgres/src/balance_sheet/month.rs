use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use async_recursion::async_recursion;
use datamize_domain::{
    async_trait,
    db::{DbError, DbResult, FinResRepo, MonthData, MonthRepo, YearData},
    Month, MonthNum, NetTotal, NetTotalType, Uuid,
};
use sqlx::PgPool;

use super::PostgresFinResRepo;

#[derive(Debug, Clone)]
pub struct PostgresMonthRepo {
    pub db_conn_pool: PgPool,
    pub fin_res_repo: PostgresFinResRepo,
}

impl PostgresMonthRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self {
            db_conn_pool: db_conn_pool.clone(),
            fin_res_repo: PostgresFinResRepo { db_conn_pool },
        })
    }

    #[tracing::instrument(skip(self, net_totals))]
    async fn insert_net_totals(&self, month_id: Uuid, net_totals: [&NetTotal; 2]) -> DbResult<()> {
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
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl MonthRepo for PostgresMonthRepo {
    #[tracing::instrument(skip(self))]
    async fn get_year_data_by_number(&self, year: i32) -> DbResult<YearData> {
        sqlx::query_as!(
            YearData,
            r#"
            SELECT id, year, refreshed_at
            FROM balance_sheet_years
            WHERE year = $1;
            "#,
            year
        )
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_month_data_by_number(&self, month: MonthNum, year: i32) -> DbResult<MonthData> {
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
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_months_of_year(&self, year: i32) -> DbResult<Vec<Month>> {
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
        .fetch_all(&self.db_conn_pool)
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
                    resources: vec![],
                });
        }

        let mut months = months.into_values().collect::<Vec<_>>();

        months.sort_by(|a, b| a.month.cmp(&b.month));

        for m in &mut months {
            m.resources = self.fin_res_repo.get_from_month(m.month, m.year).await?;
        }

        months.retain(|m| !m.resources.is_empty());

        Ok(months)
    }

    #[tracing::instrument(skip(self))]
    async fn get_months(&self) -> DbResult<Vec<Month>> {
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
        .fetch_all(&self.db_conn_pool)
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
                    resources: vec![],
                });
        }

        let mut months = months.into_values().collect::<Vec<_>>();

        months.sort_by(|a, b| match a.year.cmp(&b.year) {
            Ordering::Equal => a.month.cmp(&b.month),
            other => other,
        });

        for m in &mut months {
            m.resources = self.fin_res_repo.get_from_month(m.month, m.year).await?;
        }

        months.retain(|m| !m.resources.is_empty());

        Ok(months)
    }

    #[tracing::instrument(skip(self, month))]
    async fn add(&self, month: &Month, year: i32) -> DbResult<()> {
        let year_data = self.get_year_data_by_number(year).await?;

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_months (id, month, year_id)
            VALUES ($1, $2, $3);
            "#,
            month.id,
            month.month as i16,
            year_data.id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        self.insert_net_totals(month.id, [&month.net_assets, &month.net_portfolio])
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, month_num: MonthNum, year: i32) -> Result<Month, DbError> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                m.id as month_id,
                m.month,
                n.id as net_total_id,
                n.type,
                n.total,
                n.percent_var,
                n.balance_var
            FROM balance_sheet_months AS m
            JOIN balance_sheet_net_totals_months AS n ON m.id = n.month_id
            JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1
            WHERE m.month = $2;
            "#,
            year,
            month_num as i16,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        if db_rows.is_empty() || db_rows.len() != 2 {
            return Err(DbError::NotFound);
        }

        let resources = self.fin_res_repo.get_from_month(month_num, year).await?;

        let month: Month = Month {
            id: db_rows[0].month_id,
            month: db_rows[0].month.try_into().unwrap(),
            year,
            net_assets: db_rows
                .iter()
                .filter(|r| r.r#type == NetTotalType::Asset.to_string())
                .map(|r| NetTotal {
                    id: r.net_total_id,
                    net_type: r.r#type.parse().unwrap(),
                    total: r.total,
                    percent_var: r.percent_var,
                    balance_var: r.balance_var,
                })
                .next()
                .unwrap_or_else(NetTotal::new_asset),
            net_portfolio: db_rows
                .iter()
                .filter(|r| r.r#type == NetTotalType::Portfolio.to_string())
                .map(|r| NetTotal {
                    id: r.net_total_id,
                    net_type: r.r#type.parse().unwrap(),
                    total: r.total,
                    percent_var: r.percent_var,
                    balance_var: r.balance_var,
                })
                .next()
                .unwrap_or_else(NetTotal::new_portfolio),
            resources,
        };

        Ok(month)
    }

    #[tracing::instrument(skip(self))]
    async fn get_net_totals(&self, month_id: Uuid) -> DbResult<Vec<NetTotal>> {
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
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn update_net_totals(&self, month_num: MonthNum, year: i32) -> DbResult<()> {
        update_month_net_totals(self, month_num, year).await
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, month_num: MonthNum, year: i32) -> DbResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM balance_sheet_months
                WHERE month = $1 AND year_id in (SELECT y.id
                FROM balance_sheet_years AS y WHERE y.year = $2);
            "#,
            month_num as i16,
            year,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}

#[tracing::instrument(skip(month_repo))]
#[async_recursion]
async fn update_month_net_totals(
    month_repo: &PostgresMonthRepo,
    month_num: MonthNum,
    year: i32,
) -> DbResult<()> {
    month_repo.get_month_data_by_number(month_num, year).await?;

    let mut month = month_repo.get(month_num, year).await?;

    month.compute_net_totals();

    let prev_year = match month_num.pred() {
        MonthNum::December => year - 1,
        _ => year,
    };

    if let Ok(prev_month) = month_repo
        .get_month_data_by_number(month_num.pred(), prev_year)
        .await
    {
        if let Ok(prev_net_totals) = month_repo.get_net_totals(prev_month.id).await {
            if let Some(prev_net_assets) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Asset)
            {
                month.update_net_assets_with_previous(prev_net_assets);
            }
            if let Some(prev_net_portfolio) = prev_net_totals
                .iter()
                .find(|pnt| pnt.net_type == NetTotalType::Portfolio)
            {
                month.update_net_portfolio_with_previous(prev_net_portfolio);
            }
        }
    }

    month_repo
        .insert_net_totals(month.id, [&month.net_assets, &month.net_portfolio])
        .await?;

    let next_year_num = match month_num.succ() {
        MonthNum::January => year + 1,
        _ => year,
    };

    // Should also try to update next month if it exists
    if (month_repo.get_year_data_by_number(next_year_num).await).is_ok() {
        if let Ok(next_month) = month_repo
            .get_month_data_by_number(month_num.succ(), next_year_num)
            .await
        {
            month_repo
                .update_net_totals(next_month.month.try_into().unwrap(), next_year_num)
                .await?;
        }
    }

    Ok(())
}

// #[tracing::instrument(skip(db_conn_pool))]
// pub async fn get_months_of_resource(
//     db_conn_pool: &PgPool,
//     resource_id: Uuid,
// ) -> Result<Vec<MonthData>, sqlx::Error> {
//     let mut months: Vec<MonthData> = vec![];

//     let db_rows = sqlx::query!(
//         r#"
//             SELECT
//                 m.*
//             FROM balance_sheet_months AS m
//             JOIN balance_sheet_resources_months AS rm ON m.id = rm.month_id AND rm.resource_id = $1
//             ORDER BY m.month ASC;
//             "#,
//         resource_id,
//     )
//     .fetch_all(db_conn_pool)
//     .await?;

//     for r in db_rows {
//         months.push(MonthData {
//             id: r.id,
//             month: r.month,
//         })
//     }

//     Ok(months)
// }
