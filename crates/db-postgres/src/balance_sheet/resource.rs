use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use datamize_domain::{
    async_trait,
    db::{DbError, DbResult, FinResRepo},
    FinancialResourceMonthly, FinancialResourceYearly, MonthNum, ResourceCategory, Uuid,
    YearlyBalances,
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct PostgresFinResRepo {
    pub db_conn_pool: PgPool,
}

impl PostgresFinResRepo {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl FinResRepo for PostgresFinResRepo {
    #[tracing::instrument(skip(self))]
    async fn get_from_all_years(&self) -> DbResult<Vec<FinancialResourceYearly>> {
        let mut resources: HashMap<Uuid, FinancialResourceYearly> = HashMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.resource_id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_unique_resources AS r
            JOIN resources_balance_per_months AS rm ON r.resource_id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.month_id
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources
                .entry(r.id)
                .or_insert_with(|| {
                    FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        r.ynab_account_ids,
                        r.external_account_ids,
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
                r.resource_id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_unique_resources AS r
            JOIN resources_balance_per_months AS rm ON r.resource_id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.month_id
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id
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
                    FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        r.ynab_account_ids,
                        r.external_account_ids,
                    )
                })
                .insert_balance(r.year, r.month, r.balance);
        }

        let mut resources: Vec<FinancialResourceYearly> = resources.into_values().collect();

        resources.sort_by(|a, b| a.base.name.cmp(&b.base.name));

        Ok(resources)
    }

    async fn get_from_year_and_category(
        &self,
        year: i32,
        category: &ResourceCategory,
    ) -> DbResult<Vec<FinancialResourceYearly>> {
        let mut resources: BTreeMap<Uuid, FinancialResourceYearly> = BTreeMap::new();

        let db_rows = sqlx::query!(
            r#"
            SELECT
                r.resource_id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_unique_resources AS r
            JOIN resources_balance_per_months AS rm ON r.resource_id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.month_id
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id
            WHERE y.year = $1 AND r.resource_type LIKE '%' || $2 || '%';
            "#,
            year,
            category.to_string(),
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources
                .entry(r.id)
                .or_insert_with(|| {
                    FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        r.ynab_account_ids,
                        r.external_account_ids,
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
                    r.resource_id AS "id: Uuid",
                    r.name,
                    r.resource_type,
                    r.ynab_account_ids,
                    r.external_account_ids,
                    rm.balance
                FROM balance_sheet_unique_resources AS r
                JOIN resources_balance_per_months AS rm ON r.resource_id = rm.resource_id
                JOIN balance_sheet_months AS m ON rm.month_id = m.month_id AND m.month = $1
                JOIN balance_sheet_years AS y ON y.year_id = m.year_id AND y.year = $2
                ORDER BY r.name;
                "#,
            month as i16,
            year,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        for r in db_rows {
            resources.push(
                FinancialResourceMonthly::new(
                    r.id,
                    r.name,
                    r.resource_type.parse().unwrap(),
                    r.ynab_account_ids,
                    r.external_account_ids,
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
                r.resource_id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_unique_resources AS r
            JOIN resources_balance_per_months AS rm ON r.resource_id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.month_id
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id
            WHERE r.resource_id = $1;
            "#,
            resource_id,
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        let mut resource: Option<FinancialResourceYearly> = None;

        for r in db_rows {
            resource = Some(resource.take().map_or_else(
                || {
                    let mut res = FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        r.ynab_account_ids,
                        r.external_account_ids,
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
                r.resource_id AS "id: Uuid",
                r.name,
                r.resource_type,
                r.ynab_account_ids,
                r.external_account_ids,
                rm.balance,
                m.month AS "month: MonthNum",
                y.year AS "year: i32"
            FROM balance_sheet_unique_resources AS r
            JOIN resources_balance_per_months AS rm ON r.resource_id = rm.resource_id
            JOIN balance_sheet_months AS m ON rm.month_id = m.month_id
            JOIN balance_sheet_years AS y ON y.year_id = m.year_id
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
                    let mut res = FinancialResourceYearly::new(
                        r.id,
                        r.name,
                        r.resource_type.parse().unwrap(),
                        r.ynab_account_ids,
                        r.external_account_ids,
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
        let mut transaction = self.db_conn_pool.begin().await?;

        let resource_type = resource.base.resource_type.to_string();
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_unique_resources (resource_id, name, resource_type, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (resource_id) DO UPDATE
            SET name = EXCLUDED.name,
            resource_type = EXCLUDED.resource_type,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource_type,
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
        .execute(&mut *transaction)
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
                    m.month_id AS "id: Uuid"
                FROM balance_sheet_months AS m
                JOIN balance_sheet_years AS y ON y.year_id = m.year_id AND y.year = $1
                WHERE m.month = $2;
                "#,
                year,
                month as i16,
            )
            .fetch_one(&mut *transaction)
            .await?;

            sqlx::query!(
                r#"
                INSERT INTO resources_balance_per_months (resource_id, month_id, balance)
                VALUES ($1, $2, $3)
                ON CONFLICT (resource_id, month_id) DO UPDATE SET
                balance = EXCLUDED.balance;
                "#,
                resource.base.id,
                month_data.id,
                balance,
            )
            .execute(&mut *transaction)
            .await?;
        }

        // TODO: Check why unnest function not working...
        // let balances: Vec<(i32, i16, i64)> = resource
        //     .iter_balances()
        //     .map(|(year, month, balance)| (year, month as i16, balance))
        //     .collect();

        // sqlx::query!(
        //     r#"
        //         INSERT INTO resources_balance_per_months (resource_id, month_id, balance)
        //         SELECT r.id, m.id, x.balance
        //         FROM (
        //             SELECT unnest($1::INT[], $2::SMALLINT[], $3::BIGINT[]) AS balance
        //         ) AS x
        //         JOIN balance_sheet_unique_resources AS r ON r.id = $4
        //         JOIN balance_sheet_years AS y ON y.year = x.year
        //         JOIN balance_sheet_months AS m ON m.month = x.month AND m.year_id = y.id
        //         ON CONFLICT (resource_id, month_id) DO UPDATE SET
        //         balance = EXCLUDED.balance;
        // "#,
        //     balances.iter().map(|(year, month, balance)| *year),
        //     balances.iter().map(|(year, month, balance)| *month),
        //     balances.iter().map(|(year, month, balance)| *balance),
        //     resource.base.id,
        // )
        // .execute(&mut *transaction)
        // .await?;

        transaction.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update_and_delete(&self, resource: &FinancialResourceYearly) -> DbResult<()> {
        let mut transaction = self.db_conn_pool.begin().await?;

        let resource_type = resource.base.resource_type.to_string();
        // First update the resource itself
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_unique_resources (resource_id, name, resource_type, ynab_account_ids, external_account_ids)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (resource_id) DO UPDATE
            SET name = EXCLUDED.name,
            resource_type = EXCLUDED.resource_type,
            ynab_account_ids = EXCLUDED.ynab_account_ids,
            external_account_ids = EXCLUDED.external_account_ids;
            "#,
            resource.base.id,
            resource.base.name,
            resource_type,
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
        .execute(&mut *transaction)
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
                    m.month_id AS "id: Uuid"
                FROM balance_sheet_months AS m
                JOIN balance_sheet_years AS y ON y.year_id = m.year_id AND y.year = $1
                WHERE m.month = $2;
                "#,
                year,
                month as i16,
            )
            .fetch_one(&mut *transaction)
            .await?;

            if let Some(balance) = balance {
                sqlx::query!(
                    r#"
                    INSERT INTO resources_balance_per_months (resource_id, month_id, balance)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (resource_id, month_id) DO UPDATE SET
                    balance = EXCLUDED.balance;
                    "#,
                    resource.base.id,
                    month_data.id,
                    balance,
                )
                .execute(&mut *transaction)
                .await?;
            } else {
                sqlx::query!(
                    r#"
                    DELETE FROM resources_balance_per_months
                    WHERE resource_id = $1 AND month_id = $2;
                    "#,
                    resource.base.id,
                    month_data.id,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }
        // for (year, month, balance) in resource.iter_all_balances() {
        //     if let Some(balance) = balance {
        //         sqlx::query!(
        //             r#"
        //             INSERT INTO resources_balance_per_months (resource_id, month_id, balance)
        //             SELECT r.id, m.id, balance
        //             FROM (
        //             VALUES
        //                 ($1::bigint)
        //             ) x (balance)
        //             JOIN balance_sheet_unique_resources AS r ON r.id = $2
        //             JOIN balance_sheet_years AS y ON y.year = $3
        //             JOIN balance_sheet_months AS m ON m.month = $4 AND m.year_id = y.id
        //             ON CONFLICT (resource_id, month_id) DO UPDATE SET
        //             balance = EXCLUDED.balance;
        //             "#,
        //             balance,
        //             resource.base.id,
        //             year,
        //             month as i16,
        //         )
        //         .execute(&self.db_conn_pool)
        //         .await?;
        //     } else {
        //         sqlx::query!(
        //             r#"
        //             DELETE FROM resources_balance_per_months
        //             WHERE EXISTS(
        //                 SELECT 1 FROM balance_sheet_unique_resources AS r
        //                 JOIN balance_sheet_years AS y ON y.year = $2
        //                 JOIN balance_sheet_months AS m ON m.month = $3 AND m.year_id = y.id
        //                 WHERE r.id = $1
        //             );
        //             "#,
        //             resource.base.id,
        //             year,
        //             month as i16,
        //         )
        //         .execute(&self.db_conn_pool)
        //         .await?;
        //     }
        // }

        transaction.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, resource_id: Uuid) -> DbResult<()> {
        sqlx::query!(
            r#"
                DELETE FROM balance_sheet_unique_resources
                WHERE resource_id = $1
            "#,
            resource_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}
