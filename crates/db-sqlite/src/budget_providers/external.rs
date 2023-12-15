use datamize_domain::{
    async_trait,
    db::{external::ExternalAccountRepo, DbResult},
    secrecy::{ExposeSecret, Secret},
    AccountType, EncryptedPassword, Uuid, WebScrapingAccount,
};
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteExternalAccountRepo {
    pub db_conn_pool: SqlitePool,
}

impl SqliteExternalAccountRepo {
    pub fn new_boxed(db_conn_pool: SqlitePool) -> Box<Self> {
        Box::new(Self { db_conn_pool })
    }
}

#[async_trait]
impl ExternalAccountRepo for SqliteExternalAccountRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DbResult<Vec<WebScrapingAccount>> {
        sqlx::query!(
            r#"
            SELECT
            id as "id: Uuid",
            name,
            type as "type: AccountType",
            balance,
            username,
            encrypted_password,
            deleted
            FROM external_accounts;
            "#
        )
        .map(|row| WebScrapingAccount {
            id: row.id,
            name: row.name,
            account_type: row.r#type,
            balance: row.balance,
            username: row.username,
            encrypted_password: Secret::new(EncryptedPassword::new(row.encrypted_password)),
            deleted: row.deleted,
        })
        .fetch_all(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get(&self, account_id: Uuid) -> DbResult<WebScrapingAccount> {
        sqlx::query!(
            r#"
            SELECT
            id as "id: Uuid",
            name,
            type as "type: AccountType",
            balance,
            username,
            encrypted_password,
            deleted
            FROM external_accounts
            WHERE id = $1;
            "#,
            account_id,
        )
        .map(|row| WebScrapingAccount {
            id: row.id,
            name: row.name,
            account_type: row.r#type,
            balance: row.balance,
            username: row.username,
            encrypted_password: Secret::new(EncryptedPassword::new(row.encrypted_password)),
            deleted: row.deleted,
        })
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip(self))]
    async fn get_by_name(&self, name: &str) -> DbResult<WebScrapingAccount> {
        sqlx::query!(
            r#"
            SELECT
            id as "id: Uuid",
            name,
            type as "type: AccountType",
            balance,
            username,
            encrypted_password,
            deleted
            FROM external_accounts
            WHERE name = $1;
            "#,
            name,
        )
        .map(|row| WebScrapingAccount {
            id: row.id,
            name: row.name,
            account_type: row.r#type,
            balance: row.balance,
            username: row.username,
            encrypted_password: Secret::new(EncryptedPassword::new(row.encrypted_password)),
            deleted: row.deleted,
        })
        .fetch_one(&self.db_conn_pool)
        .await
        .map_err(Into::into)
    }

    #[tracing::instrument(skip_all)]
    async fn add(&self, account: &WebScrapingAccount) -> DbResult<()> {
        let encrypted_password = account.encrypted_password.expose_secret().as_ref();

        sqlx::query!(
            r#"
            INSERT INTO external_accounts (id, name, type, balance, username, encrypted_password, deleted)
            VALUES ($1, $2, $3, $4, $5, $6, $7);
            "#,
            account.id,
            account.name,
            account.account_type,
            account.balance,
            account.username,
            encrypted_password,
            account.deleted,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, account: &WebScrapingAccount) -> DbResult<()> {
        let encrypted_password = account.encrypted_password.expose_secret().as_ref();

        sqlx::query!(
            r#"
            INSERT INTO external_accounts (id, name, type, balance, username, encrypted_password, deleted)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            type = EXCLUDED.type,
            balance = EXCLUDED.balance,
            username = EXCLUDED.username,
            encrypted_password = EXCLUDED.encrypted_password,
            deleted = EXCLUDED.deleted;
            "#,
            account.id,
            account.name,
            account.account_type,
            account.balance,
            account.username,
            encrypted_password,
            account.deleted,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, account_id: Uuid) -> DbResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM external_accounts
            WHERE id = $1;
            "#,
            account_id,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }
}

pub async fn sabotage_external_accounts_table(pool: &SqlitePool) -> DbResult<()> {
    sqlx::query!("ALTER TABLE external_accounts DROP COLUMN name;",)
        .execute(pool)
        .await?;

    Ok(())
}
