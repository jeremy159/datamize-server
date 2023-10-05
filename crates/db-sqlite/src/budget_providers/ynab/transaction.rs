use chrono::NaiveDate;
use datamize_domain::{
    async_trait,
    db::{ynab::YnabTransactionRepo, DbResult},
    Uuid,
};
use sqlx::SqlitePool;
use ynab::{BaseTransactionDetail, ClearedType, TransactionDetail};

#[derive(Debug, Clone)]
pub struct SqliteYnabTransactionRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteYnabTransactionRepo {
    pub fn new_boxed(db_conn_pool: SqlitePool) -> Box<Self> {
        Box::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl YnabTransactionRepo for SqliteYnabTransactionRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<TransactionDetail>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT 
                id as "id: Uuid",
                date as "date: NaiveDate",
                amount,
                memo,
                cleared as "cleared: ClearedType",
                approved,
                flag_color,
                account_id as "account_id: Uuid",
                payee_id as "payee_id?: Uuid",
                category_id as "category_id?: Uuid",
                transfer_account_id as "transfer_account_id?: Uuid",
                transfer_transaction_id as "transfer_transaction_id?: Uuid",
                matched_transaction_id as "matched_transaction_id?: Uuid",
                import_id as "import_id?: Uuid",
                deleted,
                account_name,
                payee_name,
                category_name,
                subtransactions
            FROM transactions
            "#
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| TransactionDetail {
                base: BaseTransactionDetail {
                    id: r.id,
                    date: r.date,
                    amount: r.amount,
                    memo: r.memo,
                    cleared: r.cleared,
                    approved: r.approved,
                    flag_color: r.flag_color,
                    account_id: r.account_id,
                    payee_id: r.payee_id,
                    category_id: r.category_id,
                    transfer_account_id: r.transfer_account_id,
                    transfer_transaction_id: r.transfer_transaction_id,
                    matched_transaction_id: r.matched_transaction_id,
                    import_id: r.import_id,
                    deleted: r.deleted,
                    account_name: r.account_name,
                    payee_name: r.payee_name,
                    category_name: r.category_name,
                },
                subtransactions: serde_json::from_str(&r.subtransactions).unwrap(),
            })
            .collect())
    }

    #[tracing::instrument(skip_all)]
    async fn update_all(&self, transactions: &[TransactionDetail]) -> DbResult<()> {
        for t in transactions {
            let subtransactions = serde_json::to_string(&t.subtransactions).unwrap();

            sqlx::query!(
                    r#"
                    INSERT INTO transactions (id, date, amount, memo, cleared, approved, flag_color, account_id, payee_id, category_id, transfer_account_id, transfer_transaction_id, matched_transaction_id, import_id, deleted, account_name, payee_name, category_name, subtransactions)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                    ON CONFLICT (id) DO UPDATE SET
                    date = EXCLUDED.date,
                    amount = EXCLUDED.amount,
                    memo = EXCLUDED.memo,
                    cleared = EXCLUDED.cleared,
                    approved = EXCLUDED.approved,
                    flag_color = EXCLUDED.flag_color,
                    account_id = EXCLUDED.account_id,
                    payee_id = EXCLUDED.payee_id,
                    category_id = EXCLUDED.category_id,
                    transfer_account_id = EXCLUDED.transfer_account_id,
                    transfer_transaction_id = EXCLUDED.transfer_transaction_id,
                    matched_transaction_id = EXCLUDED.matched_transaction_id,
                    import_id = EXCLUDED.import_id,
                    deleted = EXCLUDED.deleted,
                    account_name = EXCLUDED.account_name,
                    payee_name = EXCLUDED.payee_name,
                    category_name = EXCLUDED.category_name,
                    subtransactions = EXCLUDED.subtransactions;
                    "#,
                    t.base.id,
                    t.base.date,
                    t.base.amount,
                    t.base.memo,
                    t.base.cleared,
                    t.base.approved,
                    t.base.flag_color,
                    t.base.account_id,
                    t.base.payee_id,
                    t.base.category_id,
                    t.base.transfer_account_id,
                    t.base.transfer_transaction_id,
                    t.base.matched_transaction_id,
                    t.base.import_id,
                    t.base.deleted,
                    t.base.account_name,
                    t.base.payee_name,
                    t.base.category_name,
                    subtransactions
                ).execute(&self.db_conn_pool).await?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_with_payee_id(&self, payee_id: Uuid) -> DbResult<Vec<TransactionDetail>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT 
                id as "id: Uuid",
                date as "date: NaiveDate",
                amount,
                memo,
                cleared as "cleared: ClearedType",
                approved,
                flag_color,
                account_id as "account_id: Uuid",
                payee_id as "payee_id?: Uuid",
                category_id as "category_id?: Uuid",
                transfer_account_id as "transfer_account_id?: Uuid",
                transfer_transaction_id as "transfer_transaction_id?: Uuid",
                matched_transaction_id as "matched_transaction_id?: Uuid",
                import_id as "import_id?: Uuid",
                deleted,
                account_name,
                payee_name,
                category_name,
                subtransactions
            FROM transactions
            WHERE payee_id = $1
            "#,
            payee_id
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| TransactionDetail {
                base: BaseTransactionDetail {
                    id: r.id,
                    date: r.date,
                    amount: r.amount,
                    memo: r.memo,
                    cleared: r.cleared,
                    approved: r.approved,
                    flag_color: r.flag_color,
                    account_id: r.account_id,
                    payee_id: r.payee_id,
                    category_id: r.category_id,
                    transfer_account_id: r.transfer_account_id,
                    transfer_transaction_id: r.transfer_transaction_id,
                    matched_transaction_id: r.matched_transaction_id,
                    import_id: r.import_id,
                    deleted: r.deleted,
                    account_name: r.account_name,
                    payee_name: r.payee_name,
                    category_name: r.category_name,
                },
                subtransactions: serde_json::from_str(&r.subtransactions).unwrap(),
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_with_category_id(
        &self,
        category_id: Uuid,
    ) -> DbResult<Vec<TransactionDetail>> {
        let db_rows = sqlx::query!(
            r#"
            SELECT 
                id as "id: Uuid",
                date as "date: NaiveDate",
                amount,
                memo,
                cleared as "cleared: ClearedType",
                approved,
                flag_color,
                account_id as "account_id: Uuid",
                payee_id as "payee_id?: Uuid",
                category_id as "category_id?: Uuid",
                transfer_account_id as "transfer_account_id?: Uuid",
                transfer_transaction_id as "transfer_transaction_id?: Uuid",
                matched_transaction_id as "matched_transaction_id?: Uuid",
                import_id as "import_id?: Uuid",
                deleted,
                account_name,
                payee_name,
                category_name,
                subtransactions
            FROM transactions
            WHERE category_id = $1
            "#,
            category_id
        )
        .fetch_all(&self.db_conn_pool)
        .await?;

        Ok(db_rows
            .into_iter()
            .map(|r| TransactionDetail {
                base: BaseTransactionDetail {
                    id: r.id,
                    date: r.date,
                    amount: r.amount,
                    memo: r.memo,
                    cleared: r.cleared,
                    approved: r.approved,
                    flag_color: r.flag_color,
                    account_id: r.account_id,
                    payee_id: r.payee_id,
                    category_id: r.category_id,
                    transfer_account_id: r.transfer_account_id,
                    transfer_transaction_id: r.transfer_transaction_id,
                    matched_transaction_id: r.matched_transaction_id,
                    import_id: r.import_id,
                    deleted: r.deleted,
                    account_name: r.account_name,
                    payee_name: r.payee_name,
                    category_name: r.category_name,
                },
                subtransactions: serde_json::from_str(&r.subtransactions).unwrap(),
            })
            .collect())
    }
}
