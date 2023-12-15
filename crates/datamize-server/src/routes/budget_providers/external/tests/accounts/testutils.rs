use axum::Router;
use datamize_domain::{
    db::external::{ExternalAccountRepo, MockEncryptionKeyRepo},
    ExternalAccount, SecretPassword, WebScrapingAccount,
};
use db_sqlite::budget_providers::external::SqliteExternalAccountRepo;
use fake::{Fake, Faker};
use sqlx::SqlitePool;

use crate::{
    routes::budget_providers::external::get_external_routes,
    services::budget_providers::ExternalAccountService,
};

pub(crate) struct TestContext {
    external_account_repo: Box<SqliteExternalAccountRepo>,
    app: Router,
}

impl TestContext {
    pub(crate) fn setup(pool: SqlitePool) -> Self {
        let external_account_repo = SqliteExternalAccountRepo::new_boxed(pool.clone());
        let encryption_key_repo = MockEncryptionKeyRepo::new_boxed();
        let external_account_service =
            ExternalAccountService::new_boxed(external_account_repo.clone(), encryption_key_repo);

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
