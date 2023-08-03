use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::DatamizeResult,
    models::budget_providers::{EncryptedPassword, WebScrapingAccount},
};

use super::ExternalAccountRepo;

#[derive(Debug, Clone)]
pub struct PostgresExternalAccountRepo {
    pub db_conn_pool: PgPool,
}

#[async_trait]
impl ExternalAccountRepo for PostgresExternalAccountRepo {
    #[tracing::instrument(skip(self))]
    async fn get_all(&self) -> DatamizeResult<Vec<WebScrapingAccount>> {
        sqlx::query!(
            r#"
            SELECT * FROM external_accounts;
            "#
        )
        .map(|row| WebScrapingAccount {
            id: row.id,
            name: row.name,
            account_type: row.r#type.parse().unwrap(),
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
    async fn get(&self, account_id: Uuid) -> DatamizeResult<WebScrapingAccount> {
        sqlx::query!(
            r#"
            SELECT * FROM external_accounts
            WHERE id = $1;
            "#,
            account_id,
        )
        .map(|row| WebScrapingAccount {
            id: row.id,
            name: row.name,
            account_type: row.r#type.parse().unwrap(),
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
    async fn get_by_name(&self, name: &str) -> DatamizeResult<WebScrapingAccount> {
        sqlx::query!(
            r#"
            SELECT * FROM external_accounts
            WHERE name = $1;
            "#,
            name,
        )
        .map(|row| WebScrapingAccount {
            id: row.id,
            name: row.name,
            account_type: row.r#type.parse().unwrap(),
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
    async fn add(&self, account: &WebScrapingAccount) -> DatamizeResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO external_accounts (id, name, type, balance, username, encrypted_password, deleted)
            VALUES ($1, $2, $3, $4, $5, $6, $7);
            "#,
            account.id,
            account.name,
            account.account_type.to_string(),
            account.balance,
            account.username,
            account.encrypted_password.expose_secret().as_ref(),
            account.deleted,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn update(&self, account: &WebScrapingAccount) -> DatamizeResult<()> {
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
            account.account_type.to_string(),
            account.balance,
            account.username,
            account.encrypted_password.expose_secret().as_ref(),
            account.deleted,
        )
        .execute(&self.db_conn_pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, account_id: Uuid) -> DatamizeResult<()> {
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
