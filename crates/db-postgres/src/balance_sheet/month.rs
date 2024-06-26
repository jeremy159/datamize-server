use itertools::Itertools;
use std::{cell::RefCell, rc::Rc, sync::Arc};

use chrono::{DateTime, Utc};
use datamize_domain::{
    async_trait,
    db::{DbError, DbResult, FinResRepo, MonthData, MonthRepo, NetTotalType, YearData},
    Month, MonthNum, NetTotal, NetTotals, Uuid,
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
}

#[async_trait]
impl MonthRepo for PostgresMonthRepo {
    #[tracing::instrument(skip(self))]
    async fn get_year_data_by_number(&self, year: i32) -> DbResult<YearData> {
        sqlx::query_as!(
            YearData,
            r#"
            SELECT year_id AS "id: Uuid", year, refreshed_at
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
                m.month_id AS "id: Uuid",
                m.month as "month: MonthNum",
                y.year as "year: i32"
            FROM balance_sheet_months AS m
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id AND y.year = $1
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
    async fn get_months_of_year_without_resources(&self, year: i32) -> DbResult<Vec<Month>> {
        let month_datas = sqlx::query_as!(
            MonthData,
            r#"
            SELECT
                m.month_id as "id: Uuid",
                m.month as "month: MonthNum",
                y.year as "year: i32"
            FROM balance_sheet_months AS m
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id AND y.year = $1
            ORDER BY m.month;
            "#,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut months: Vec<Month> = vec![];

        for md in month_datas {
            let net_totals = self.get_net_totals(md.id).await?;

            months.push(Month {
                id: md.id,
                month: md.month,
                year: md.year,
                net_totals,
                resources: vec![],
            });
        }

        Ok(months)
    }

    #[tracing::instrument(skip(self))]
    async fn get_months_of_year(&self, year: i32) -> DbResult<Vec<Month>> {
        let mut months = self.get_months_of_year_without_resources(year).await?;

        for m in &mut months {
            m.resources = self.fin_res_repo.get_from_month(m.month, m.year).await?;
        }

        // Filter out months with no resources
        months.retain(|m| !m.resources.is_empty());

        Ok(months)
    }

    #[tracing::instrument(skip(self))]
    async fn get_months(&self) -> DbResult<Vec<Month>> {
        let month_datas = sqlx::query_as!(
            MonthData,
            r#"
            SELECT
                m.month_id as "id: Uuid",
                m.month as "month: MonthNum",
                y.year as "year: i32"
            FROM balance_sheet_months AS m
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id
            ORDER BY y.year, m.month;
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut months: Vec<Month> = vec![];

        for md in month_datas {
            let net_totals = self.get_net_totals(md.id).await?;
            let resources = self.fin_res_repo.get_from_month(md.month, md.year).await?;

            months.push(Month {
                id: md.id,
                month: md.month,
                year: md.year,
                net_totals,
                resources,
            });
        }

        // Filter out months with no resources
        months.retain(|m| !m.resources.is_empty());

        Ok(months)
    }

    #[tracing::instrument(skip(self))]
    async fn get_months_starting_from(
        &self,
        month_num: MonthNum,
        year: i32,
    ) -> DbResult<Vec<Month>> {
        let month_datas = sqlx::query_as!(
            MonthData,
            r#"
            SELECT
                m.month_id as "id: Uuid",
                m.month as "month: MonthNum",
                y.year as "year: i32"
            FROM balance_sheet_months AS m
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id
            WHERE (y.year > $2 OR (y.year = $2 AND m.month >= $1))
            ORDER BY y.year, m.month;
            "#,
            month_num as i16,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut months: Vec<Month> = vec![];

        for md in month_datas {
            let net_totals = self.get_net_totals(md.id).await?;
            let resources = self.fin_res_repo.get_from_month(md.month, md.year).await?;

            months.push(Month {
                id: md.id,
                month: md.month,
                year: md.year,
                net_totals,
                resources,
            });
        }

        Ok(months)
    }

    #[tracing::instrument(skip(self, month))]
    async fn add(&self, month: &Month, year: i32) -> DbResult<()> {
        let year_data = self.get_year_data_by_number(year).await?;

        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_months (month_id, month, year_id)
            VALUES ($1, $2, $3);
            "#,
            month.id,
            month.month as i16,
            year_data.id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        self.insert_net_totals(month.id, &month.net_totals).await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_without_resources(&self, month_num: MonthNum, year: i32) -> DbResult<Month> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                m.month_id as "month_id: Uuid",
                m.month as "month: MonthNum",
                n.net_total_id as "net_total_id: Uuid",
                n.type as "net_type: NetTotalType",
                n.total,
                n.percent_var as "percent_var: f32",
                n.balance_var,
                n.last_updated as "last_updated?: DateTime<Utc>"
            FROM balance_sheet_months AS m
            JOIN balance_sheet_net_totals_months AS n ON m.month_id = n.month_id
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id AND y.year = $1
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

        let id = db_rows[0].month_id;
        let month = db_rows[0].month;
        let mut net_totals = NetTotals::default();

        for r in db_rows {
            match r.net_type {
                NetTotalType::Asset => {
                    net_totals.assets = NetTotal {
                        id: r.net_total_id,
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                        last_updated: r.last_updated,
                    };
                }
                NetTotalType::Portfolio => {
                    net_totals.portfolio = NetTotal {
                        id: r.net_total_id,
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                        last_updated: r.last_updated,
                    };
                }
            };
        }

        let month = Month {
            id,
            month,
            year,
            net_totals,
            resources: vec![],
        };

        Ok(month)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, month_num: MonthNum, year: i32) -> Result<Month, DbError> {
        let mut month = self.get_without_resources(month_num, year).await?;
        month.resources = self.fin_res_repo.get_from_month(month_num, year).await?;

        Ok(month)
    }

    #[tracing::instrument(skip(self))]
    async fn get_net_totals(&self, month_id: Uuid) -> DbResult<NetTotals> {
        let rows = sqlx::query!(
            r#"
            SELECT
                net_total_id AS "id: Uuid",
                type AS "net_type: NetTotalType",
                total,
                percent_var as "percent_var: f32",
                balance_var,
                last_updated as "last_updated?: DateTime<Utc>"
            FROM balance_sheet_net_totals_months
            WHERE month_id = $1;
            "#,
            month_id,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut net_totals = NetTotals::default();

        for r in rows {
            match r.net_type {
                NetTotalType::Asset => {
                    net_totals.assets = NetTotal {
                        id: r.id,
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                        last_updated: r.last_updated,
                    };
                }
                NetTotalType::Portfolio => {
                    net_totals.portfolio = NetTotal {
                        id: r.id,
                        total: r.total,
                        percent_var: r.percent_var,
                        balance_var: r.balance_var,
                        last_updated: r.last_updated,
                    };
                }
            };
        }

        Ok(net_totals)
    }

    #[tracing::instrument(skip(self))]
    async fn update_net_totals(&self, month_num: MonthNum, year: i32) -> DbResult<()> {
        let mut months = self.get_months_starting_from(month_num, year).await?;
        if let Some(first_month) = months.first_mut() {
            first_month.compute_net_totals();
            let prev_year = match month_num.pred() {
                MonthNum::December => year - 1,
                _ => year,
            };

            if let Ok(prev_month) = self
                .get_without_resources(month_num.pred(), prev_year)
                .await
            {
                first_month.compute_variation(&prev_month);
            }
        }

        for (prev_month, curr_month) in months
            .iter_mut()
            .map(RefCell::new)
            .map(Rc::new)
            .tuple_windows()
        {
            let mut curr_month = curr_month.borrow_mut();
            curr_month.compute_net_totals();
            curr_month.compute_variation(&prev_month.borrow());
        }

        for month in months {
            self.insert_net_totals(month.id, &month.net_totals).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, net_totals))]
    async fn insert_net_totals(&self, month_id: Uuid, net_totals: &NetTotals) -> DbResult<()> {
        let mut transaction = self.db_conn_pool.begin().await?;

        let net_type = NetTotalType::Asset.to_string();
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_months (net_total_id, type, total, percent_var, balance_var, last_updated, month_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (net_total_id) DO UPDATE
            SET type = EXCLUDED.type,
            total = EXCLUDED.total,
            percent_var = EXCLUDED.percent_var,
            balance_var = EXCLUDED.balance_var,
            last_updated = EXCLUDED.last_updated;
            "#,
            net_totals.assets.id,
            net_type,
            net_totals.assets.total,
            net_totals.assets.percent_var,
            net_totals.assets.balance_var,
            net_totals.assets.last_updated,
            month_id,
        )
        .execute(&mut *transaction)
        .await?;

        let net_type = NetTotalType::Portfolio.to_string();
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_months (net_total_id, type, total, percent_var, balance_var, last_updated, month_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (net_total_id) DO UPDATE
            SET type = EXCLUDED.type,
            total = EXCLUDED.total,
            percent_var = EXCLUDED.percent_var,
            balance_var = EXCLUDED.balance_var,
            last_updated = EXCLUDED.last_updated;
            "#,
            net_totals.portfolio.id,
            net_type,
            net_totals.portfolio.total,
            net_totals.portfolio.percent_var,
            net_totals.portfolio.balance_var,
            net_totals.portfolio.last_updated,
            month_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, month_num: MonthNum, year: i32) -> DbResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM balance_sheet_months
                WHERE month = $1 AND year_id in (SELECT y.year_id
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
