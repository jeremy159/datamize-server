use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use datamize_domain::{
    async_trait,
    db::{DbError, DbResult, FinResRepo},
    FinancialResourceMonthly, FinancialResourceYearly, MonthNum, UpdateResource, Uuid,
    YearlyBalances,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteFinResRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteFinResRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }

    #[tracing::instrument(skip_all)]
    pub async fn update_monthly(
        &self,
        resource: &FinancialResourceMonthly,
        month: MonthNum,
        year: i32,
    ) -> DbResult<()> {
        let ynab_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.ynab_account_ids.clone(),
        })
        .unwrap();
        let external_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.external_account_ids.clone(),
        })
        .unwrap();

        let resource_type = resource.base.resource_type.to_string();
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, resource_type, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            resource_type = EXCLUDED.resource_type,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource_type,
            ynab_account_ids,
            external_account_ids,
        )
        .execute(&self.db_conn_pool)
        .await?;

        #[derive(Debug)]
        struct MonthData {
            id: Uuid,
        }

        let month_data = sqlx::query_as!(
            MonthData,
            r#"
            SELECT
                m.id AS "id: Uuid"
            FROM balance_sheet_months AS m
            JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1
            WHERE m.month = $2;
            "#,
            year,
            month,
        )
        .fetch_one(&self.db_conn_pool)
        .await?;

        // Then the balance of the month
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
            VALUES ($1, $2, $3)
            ON CONFLICT (resource_id, month_id) DO UPDATE SET
            balance = EXCLUDED.balance;
            "#,
            resource.base.id,
            month_data.id,
            resource.balance,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl FinResRepo for SqliteFinResRepo {
    #[tracing::instrument(skip(self))]
    async fn get_from_all_years(&self) -> DbResult<Vec<FinancialResourceYearly>> {
        let mut resources: HashMap<Uuid, FinancialResourceYearly> = HashMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
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
                .or_insert_with(|| {
                    let ynab_account_ids = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;
                    let external_account_ids = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;

                    FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        ynab_account_ids,
                        external_account_ids,
                    )
                })
                .insert_balance(r.year, r.month, r.balance);
        }

        let mut resources: Vec<FinancialResourceYearly> = resources.into_values().collect();

        resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_year(&self, year: i32) -> DbResult<Vec<FinancialResourceYearly>> {
        let mut resources: BTreeMap<Uuid, FinancialResourceYearly> = BTreeMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id
            WHERE y.year = $1;
            "#,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources
                .entry(r.id)
                .or_insert_with(|| {
                    let ynab_account_ids = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;
                    let external_account_ids = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;

                    FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        ynab_account_ids,
                        external_account_ids,
                    )
                })
                .insert_balance(r.year, r.month, r.balance);
        }

        let mut resources: Vec<FinancialResourceYearly> = resources.into_values().collect();

        resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));

        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get_from_month(
        &self,
        month: MonthNum,
        year: i32,
    ) -> DbResult<Vec<FinancialResourceMonthly>> {
        let mut resources: Vec<FinancialResourceMonthly> = vec![];

        let db_rows = sqlx::query!(
            r#"
                SELECT
                    r.id AS "id: Uuid",
                    r.name,
                    r.resource_type,
                    r.ynab_account_ids,
                    r.external_account_ids,
                    rm.balance
                FROM balance_sheet_resources AS r
                JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
                JOIN balance_sheet_months AS m ON rm.month_id = m.id AND m.month = $1
                JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $2
                ORDER BY r.name;
                "#,
            month,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            let ynab_account_ids = r
                .ynab_account_ids
                .as_ref()
                .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                .unwrap_or(IdsRecord { ids: None })
                .ids;
            let external_account_ids = r
                .external_account_ids
                .as_ref()
                .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                .unwrap_or(IdsRecord { ids: None })
                .ids;

            resources.push(
                FinancialResourceMonthly::new(
                    r.id,
                    r.name,
                    r.resource_type.parse().unwrap(),
                    ynab_account_ids,
                    external_account_ids,
                )
                .with_balance(r.balance),
            );
        }
        Ok(resources)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, resource_id: Uuid) -> DbResult<FinancialResourceYearly> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id
            WHERE r.id = $1;
            "#,
            resource_id,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut resource: Option<FinancialResourceYearly> = None;

        for r in db_rows {
            resource = Some(resource.take().map_or_else(
                || {
                    let ynab_account_ids = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;
                    let external_account_ids = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;

                    let mut res = FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        ynab_account_ids,
                        external_account_ids,
                    );
                    res.insert_balance(r.year, r.month, r.balance);
                    res
                },
                |mut res| {
                    res.insert_balance(r.year, r.month, r.balance);
                    res
                },
            ));
        }

        resource.ok_or(DbError::NotFound)
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_name(&self, name: &str) -> DbResult<FinancialResourceYearly> {
        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_resources AS r
            JOIN balance_sheet_resources_months AS rm ON r.id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.id
            JOIN balance_sheet_years AS y ON y.id = m.year_id
            WHERE r.name = $1;
            "#,
            name,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut resource: Option<FinancialResourceYearly> = None;

        for r in db_rows {
            resource = Some(resource.take().map_or_else(
                || {
                    let ynab_account_ids = r
                        .ynab_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;
                    let external_account_ids = r
                        .external_account_ids
                        .as_ref()
                        .map(|r| serde_json::from_str::<IdsRecord>(r).unwrap())
                        .unwrap_or(IdsRecord { ids: None })
                        .ids;

                    let mut res = FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        ynab_account_ids,
                        external_account_ids,
                    );
                    res.insert_balance(r.year, r.month, r.balance);
                    res
                },
                |mut res| {
                    res.insert_balance(r.year, r.month, r.balance);
                    res
                },
            ));
        }

        resource.ok_or(DbError::NotFound)
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, resource: &FinancialResourceYearly) -> DbResult<()> {
        let ynab_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.ynab_account_ids.clone(),
        })
        .unwrap();
        let external_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.external_account_ids.clone(),
        })
        .unwrap();

        let resource_type = resource.base.resource_type.to_string();
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, resource_type, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            resource_type = EXCLUDED.resource_type,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource_type,
            ynab_account_ids,
            external_account_ids,
        )
        .execute(&self.db_conn_pool)
        .await?;
        #[derive(Debug)]
        struct MonthData {
            id: Uuid,
        }

        // Then the balance per month
        for (year, month, balance) in resource.iter_balances() {
            let month_data = sqlx::query_as!(
                MonthData,
                r#"
                SELECT
                    m.id AS "id: Uuid"
                FROM balance_sheet_months AS m
                JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1
                WHERE m.month = $2;
                "#,
                year,
                month,
            )
            .fetch_one(&self.db_conn_pool)
            .await?;

            // Then the balance of the month
            sqlx::query!(
                r#"
                INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
                VALUES ($1, $2, $3)
                ON CONFLICT (resource_id, month_id) DO UPDATE SET
                balance = EXCLUDED.balance;
                "#,
                resource.base.id,
                month_data.id,
                balance,
            )
            .execute(&self.db_conn_pool)
            .await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update_and_delete(&self, resource: &UpdateResource) -> DbResult<()> {
        let ynab_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.ynab_account_ids.clone(),
        })
        .unwrap();
        let external_account_ids = serde_json::to_string(&IdsRecord {
            ids: resource.base.external_account_ids.clone(),
        })
        .unwrap();

        let resource_type = resource.base.resource_type.to_string();
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_resources (id, name, resource_type, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
            resource_type = EXCLUDED.resource_type,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource_type,
            ynab_account_ids,
            external_account_ids,
        )
        .execute(&self.db_conn_pool)
        .await?;
        #[derive(Debug)]
        struct MonthData {
            id: Uuid,
        }

        // Then the balance per month
        for (year, month, balance) in resource.iter_all_balances() {
            let month_data = sqlx::query_as!(
                MonthData,
                r#"
                SELECT
                    m.id AS "id: Uuid"
                FROM balance_sheet_months AS m
                JOIN balance_sheet_years AS y ON y.id = m.year_id AND y.year = $1
                WHERE m.month = $2;
                "#,
                year,
                month,
            )
            .fetch_one(&self.db_conn_pool)
            .await?;

            if let Some(balance) = balance {
                sqlx::query!(
                    r#"
                    INSERT INTO balance_sheet_resources_months (resource_id, month_id, balance)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (resource_id, month_id) DO UPDATE SET
                    balance = EXCLUDED.balance;
                    "#,
                    resource.base.id,
                    month_data.id,
                    balance,
                )
                .execute(&self.db_conn_pool)
                .await?;
            } else {
                sqlx::query!(
                    r#"
                    DELETE FROM balance_sheet_resources_months
                    WHERE resource_id = $1 AND month_id = $2;
                    "#,
                    resource.base.id,
                    month_data.id,
                )
                .execute(&self.db_conn_pool)
                .await?;
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, resource_id: Uuid) -> DbResult<()> {
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

#[derive(Debug, Serialize, Deserialize)]
struct IdsRecord {
    ids: Option<Vec<Uuid>>,
}

pub async fn sabotage_resources_table(pool: &SqlitePool) -> DbResult<()> {
    sqlx::query!("ALTER TABLE balance_sheet_resources DROP COLUMN name;",)
        .execute(pool)
        .await?;

    Ok(())
}
