use std::sync::Arc;

use chrono::{DateTime, Utc};
use datamize_domain::{
    async_trait,
    db::{ynab::YnabAccountRepo, DbResult},
    Uuid,
};
use sqlx::SqlitePool;
use ynab::{Account, AccountType};

#[derive(Debug, Clone)]
pub struct SqliteYnabAccountRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteYnabAccountRepo {
    pub fn new_arced(db_conn_pool: SqlitePool) -> Arc<Self> {
        Arc::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl YnabAccountRepo for SqliteYnabAccountRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<Account>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id as "id: Uuid",
                name,
                type AS "account_type: AccountType",
                on_budget,
                closed,
                note,
                balance,
                cleared_balance,
                uncleared_balance,
                transfer_payee_id as "transfer_payee_id: Uuid",
                direct_import_linked,
                direct_import_in_error,
                deleted,
                last_reconciled_at as "last_reconciled_at: DateTime<Utc>",
                debt_original_balance,
                debt_interest_rates,
                debt_minimum_payments,
                debt_escrow_amounts
            FROM accounts
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Account {
                id: r.id,
                name: r.name,
                account_type: r.account_type,
                on_budget: r.on_budget,
                closed: r.closed,
                note: r.note,
                balance: r.balance,
                cleared_balance: r.cleared_balance,
                uncleared_balance: r.uncleared_balance,
                transfer_payee_id: r.transfer_payee_id,
                direct_import_linked: r.direct_import_linked,
                direct_import_in_error: r.direct_import_in_error,
                deleted: r.deleted,
                last_reconciled_at: r.last_reconciled_at,
                debt_original_balance: r.debt_original_balance,
                debt_interest_rates: serde_json::from_str(&r.debt_interest_rates).unwrap(),
                debt_minimum_payments: serde_json::from_str(&r.debt_minimum_payments).unwrap(),
                debt_escrow_amounts: serde_json::from_str(&r.debt_escrow_amounts).unwrap(),
            })
            .collect())
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, accounts: &[Account]) -> DbResult<()> {
        for a in accounts {
            let debt_interest_rates = serde_json::to_string(&a.debt_interest_rates).unwrap();
            let debt_minimum_payments = serde_json::to_string(&a.debt_minimum_payments).unwrap();
            let debt_escrow_amounts = serde_json::to_string(&a.debt_escrow_amounts).unwrap();

            sqlx::query!(
                    r#"
                    INSERT INTO accounts (id, name, type, on_budget, closed, note, balance, cleared_balance, uncleared_balance, transfer_payee_id, direct_import_linked, direct_import_in_error, deleted, last_reconciled_at, debt_original_balance, debt_interest_rates, debt_minimum_payments, debt_escrow_amounts)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                    ON CONFLICT (id) DO UPDATE SET
                    name = EXCLUDED.name,
                    type = EXCLUDED.type,
                    on_budget = EXCLUDED.on_budget,
                    closed = EXCLUDED.closed,
                    note = EXCLUDED.note,
                    balance = EXCLUDED.balance,
                    cleared_balance = EXCLUDED.cleared_balance,
                    uncleared_balance = EXCLUDED.uncleared_balance,
                    transfer_payee_id = EXCLUDED.transfer_payee_id,
                    direct_import_linked = EXCLUDED.direct_import_linked,
                    direct_import_in_error = EXCLUDED.direct_import_in_error,
                    deleted = EXCLUDED.deleted,
                    last_reconciled_at = EXCLUDED.last_reconciled_at,
                    debt_original_balance = EXCLUDED.debt_original_balance,
                    debt_interest_rates = EXCLUDED.debt_interest_rates,
                    debt_minimum_payments = EXCLUDED.debt_minimum_payments,
                    debt_escrow_amounts = EXCLUDED.debt_escrow_amounts;
                    "#,
                    a.id,
                    a.name,
                    a.account_type,
                    a.on_budget,
                    a.closed,
                    a.note,
                    a.balance,
                    a.cleared_balance,
                    a.uncleared_balance,
                    a.transfer_payee_id,
                    a.direct_import_linked,
                    a.direct_import_in_error,
                    a.deleted,
                    a.last_reconciled_at,
                    a.debt_original_balance,
                    debt_interest_rates,
                    debt_minimum_payments,
                    debt_escrow_amounts,
                    
                ).execute(&self.db_conn_pool).await?;
        }

        Ok(())
    }
}
