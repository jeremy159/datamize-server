use std::collections::{BTreeMap, HashMap};

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    db::balance_sheet::FinResRepo,
    error::{AppError, DatamizeResult},
    models::balance_sheet::{
        BaseFinancialResource, FinancialResourceMonthly, FinancialResourceYearly, MonthNum,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresFinResRepo {
    pub db_conn_pool: PgPool,
}

impl PostgresFinResRepo {
    pub fn new(db_conn_pool: PgPool) -> Self {
        Self { db_conn_pool }
    }
}

#[async_trait]
impl FinResRepo for PostgresFinResRepo {
    #[tracing::instrument(skip(self))]
    async fn get_from_all_years(&self) -> DatamizeResult<Vec<FinancialResourceYearly>> {
        let mut resources: HashMap<Uuid, FinancialResourceYearly> = HashMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.*,
                rm.balance,
                m.month,
                y.year
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources
                .entry(r.id)
                .and_modify(|res| {
                    res.balance_per_month
                        .insert(r.month.try_into().unwrap(), r.balance);
                })
                .or_insert_with(|| {
                    let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                    // Relations in the DB enforces that only one month in a year exists for one resource
                    balance_per_month.insert(r.month.try_into().unwrap(), r.balance);

                    FinancialResourceYearly {
                        base: BaseFinancialResource {
                            id: r.id,
                            name: r.name,
                            category: r.category.parse().unwrap(),
                            r_type: r.r#type.parse().unwrap(),
                            editable: r.editable,
                            ynab_account_ids: r.ynab_account_ids,
                            external_account_ids: r.external_account_ids,
                        },
                        year: r.year,
                        balance_per_month,
                    }
                });
        }

        let mut resources: Vec<FinancialResourceYearly> = resources.into_values().collect();

        resources.sort_by(|a, b| a.year.cmp(&b.year));

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_year(&self, year: i32) -> DatamizeResult<Vec<FinancialResourceYearly>> {
        let mut resources: BTreeMap<Uuid, FinancialResourceYearly> = BTreeMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.*,
                rm.balance,
                m.month
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1;
            "#,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources
                .entry(r.id)
                .and_modify(|res| {
                    res.balance_per_month
                        .insert(r.month.try_into().unwrap(), r.balance);
                })
                .or_insert_with(|| {
                    let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                    // Relations in the DB enforces that only one month in a year exists for one resource
                    balance_per_month.insert(r.month.try_into().unwrap(), r.balance);

                    FinancialResourceYearly {
                        base: BaseFinancialResource {
                            id: r.id,
                            name: r.name,
                            category: r.category.parse().unwrap(),
                            r_type: r.r#type.parse().unwrap(),
                            editable: r.editable,
                            ynab_account_ids: r.ynab_account_ids,
                            external_account_ids: r.external_account_ids,
                        },
                        year,
                        balance_per_month,
                    }
                });
        }

        Ok(resources.into_values().collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_month(
        &self,
        month: MonthNum,
        year: i32,
    ) -> DatamizeResult<Vec<FinancialResourceMonthly>> {
        let mut resources: Vec<FinancialResourceMonthly> = vec![];

        let db_rows = sqlx::query!(
            r#"
                SELECT
                    r.*,
                    rm.balance,
                    m.month,
                    y.year
                FROM balance_sheet_resources AS r
                JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
                JOIN balance_sheet_months AS m ON rm.month_id = m.id AND m.month = $1
                JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $2;
                "#,
            month as i16,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources.push(FinancialResourceMonthly {
                base: BaseFinancialResource {
                    id: r.id,
                    name: r.name,
                    category: r.category.parse().unwrap(),
                    r_type: r.r#type.parse().unwrap(),
                    editable: r.editable,
                    ynab_account_ids: r.ynab_account_ids,
                    external_account_ids: r.external_account_ids,
                },
                month: r.month.try_into().unwrap(),
                year: r.year,
                balance: r.balance,
            });
        }

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, resource_id: Uuid) -> DatamizeResult<FinancialResourceYearly> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.*,
                rm.balance,
                m.month,
                y.year
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id AND r.id = $1
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id;
            "#,
            resource_id,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut resource: Option<FinancialResourceYearly> = None;

        for r in db_rows {
            match resource {
                Some(ref mut res) => {
                    res.balance_per_month
                        .insert(r.month.try_into().unwrap(), r.balance);
                }
                None => {
                    let mut balance_per_month: BTreeMap<MonthNum, i64> = BTreeMap::new();

                    // Relations in the DB enforces that only one month in a year exists for one resource
                    balance_per_month.insert(r.month.try_into().unwrap(), r.balance);

                    resource = Some(FinancialResourceYearly {
                        base: BaseFinancialResource {
                            id: r.id,
                            name: r.name,
                            category: r.category.parse().unwrap(),
                            r_type: r.r#type.parse().unwrap(),
                            editable: r.editable,
                            ynab_account_ids: r.ynab_account_ids,
                            external_account_ids: r.external_account_ids,
                        },
                        year: r.year,
                        balance_per_month,
                    })
                }
            }
        }

        resource.ok_or(AppError::ResourceNotFound)
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, resource: &FinancialResourceYearly) -> DatamizeResult<()> {
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, category, type, editable, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            category = EXCLUDED.category,
            type = EXCLUDED.type,
            editable = EXCLUDED.editable,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource.base.category.to_string(),
            resource.base.r_type.to_string(),
            resource.base.editable,
            resource
                .base
                .ynab_account_ids
                .as_ref()
                .map(|accounts| accounts.as_slice()),
                resource
                .base
                .external_account_ids
                .as_ref()
                .map(|accounts| accounts.as_slice()),
        )
        .execute(&self.db_conn_pool)
        .await?;

        // Then the balance per month
        for (month, balance) in &resource.balance_per_month {
            sqlx::query!(
                r#"
                INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
                SELECT r.id, m.id, balance
                FROM (
                VALUES
                    ($1::bigint)
                ) x (balance)
                JOIN balance_sheet_resources AS r ON r.id = $2
                JOIN balance_sheet_years AS y ON y.year = $3
                JOIN balance_sheet_months AS m ON m.month = $4 AND m.year_id = y.id
                ON CONFLICT (resource_id, month_id) DO UPDATE SET
                balance = EXCLUDED.balance;
                "#,
                balance,
                resource.base.id,
                resource.year,
                *month as i16,
            )
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update_monthly(&self, resource: &FinancialResourceMonthly) -> DatamizeResult<()> {
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, category, type, editable)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            category = EXCLUDED.category,
            type = EXCLUDED.type,
            editable = EXCLUDED.editable;
            "#,
            resource.base.id,
            resource.base.name,
            resource.base.category.to_string(),
            resource.base.r_type.to_string(),
            resource.base.editable,
        )
        .execute(&self.db_conn_pool)
        .await?;

        // Then the balance of the month
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
            SELECT r.id, m.id, balance
            FROM (
            VALUES
                ($1::bigint)
            ) x (balance)
            JOIN balance_sheet_resources AS r ON r.id = $2
            JOIN balance_sheet_years AS y ON y.year = $3
            JOIN balance_sheet_months AS m ON m.month = $4 AND m.year_id = y.id
            ON CONFLICT (resource_id, month_id) DO UPDATE SET
            balance = EXCLUDED.balance;
            "#,
            resource.balance,
            resource.base.id,
            resource.year,
            resource.month as i16,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, resource_id: Uuid) -> DatamizeResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM balance_sheet_resources
                WHERE id = $1
            "#,
            resource_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}
