use std::sync::Arc;

use axum::Router;
use datamize_domain::{
    db::external::{EncryptionKeyRepo, ExternalAccountRepo},
    ExternalAccount, SecretPassword, WebScrapingAccount,
};
use db_redis::{budget_providers::external::RedisEncryptionKeyRepo, get_test_pool};
use db_sqlite::budget_providers::external::SqliteExternalAccountRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;

use crate::{
    routes::api::budget_providers::external::get_external_routes,
    services::budget_providers::ExternalAccountService,
};

pub(crate) struct TestContext {
    external_account_repo: Arc<SqliteExternalAccountRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) async fn setup(pool: SqlitePool) -> Self {
        let redis_conn_pool = get_test_pool().await;
        let external_account_repo = SqliteExternalAccountRepo::new_arced(pool.clone());
        let encryption_key_repo = RedisEncryptionKeyRepo::new_arced(redis_conn_pool);
        encryption_key_repo.set(&fake::vec![u8; 6]).await.unwrap();
        let external_account_service =
            ExternalAccountService::new_arced(external_account_repo.clone(), encryption_key_repo);

        let app = get_external_routes(external_account_service);
        Self {
            external_account_repo,
            app,
        }
    }

    pub(crate) fn into_app(self) -> Router {
        self.app
    }

    pub(crate) async fn set_accounts(&self, accounts: &[WebScrapingAccount]) {
        for a in accounts {
            self.external_account_repo.update(a).await.unwrap();
        }
    }
}

pub(crate) fn correctly_stub_accounts(accounts: Vec<ExternalAccount>) -> Vec<WebScrapingAccount> {
    accounts
        .into_iter()
        .map(|a| WebScrapingAccount {
            id: a.id,
            name: a.name,
            account_type: a.account_type,
            balance: a.balance,
            deleted: a.deleted,
            username: Faker.fake(),
            encrypted_password: SecretPassword::new(Faker.fake()),
        })
        .collect()
}
