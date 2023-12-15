use std::sync::Arc;

use anyhow::Context;
use datamize_domain::{
    async_trait,
    db::ynab::{DynYnabAccountMetaRepo, DynYnabAccountRepo},
};
use dyn_clone::{clone_trait_object, DynClone};
use ynab::{Account, AccountRequests};

use crate::error::DatamizeResult;

#[async_trait]
pub trait YnabAccountServiceExt: DynClone + Send + Sync {
    async fn get_all_ynab_accounts(&mut self) -> DatamizeResult<Vec<Account>>;
}

clone_trait_object!(YnabAccountServiceExt);

pub type DynYnabAccountService = Box<dyn YnabAccountServiceExt>;

#[derive(Clone)]
pub struct YnabAccountService {
    pub ynab_account_repo: DynYnabAccountRepo,
    pub ynab_account_meta_repo: DynYnabAccountMetaRepo,
    pub ynab_client: Arc<dyn AccountRequests + Send + Sync>,
}

#[async_trait]
impl YnabAccountServiceExt for YnabAccountService {
    #[tracing::instrument(skip(self))]
    async fn get_all_ynab_accounts(&mut self) -> DatamizeResult<Vec<Account>> {
        let saved_accounts_delta = self.ynab_account_meta_repo.get_delta().await.ok();

        let accounts_delta = self
            .ynab_client
            .get_accounts_delta(saved_accounts_delta)
            .await
            .context("failed to get accounts from ynab's API")?;

        let accounts = accounts_delta
            .accounts
            .into_iter()
            .filter(|a| !a.deleted)
            .collect::<Vec<_>>();

        self.ynab_account_repo
            .update_all(&accounts)
            .await
            .context("failed to save accounts in database")?;

        self.ynab_account_meta_repo
            .set_delta(accounts_delta.server_knowledge)
            .await
            .context("failed to save last known server knowledge of accounts in redis")?;

        let saved_accounts = self
            .ynab_account_repo
            .get_all()
            .await
            .context("failed to get accounts from database")?;

        Ok(saved_accounts)
    }
}

impl YnabAccountService {
    pub fn new_boxed(
        ynab_account_repo: DynYnabAccountRepo,
        ynab_account_meta_repo: DynYnabAccountMetaRepo,
        ynab_client: Arc<dyn AccountRequests + Send + Sync>,
    ) -> Box<Self> {
        Box::new(YnabAccountService {
            ynab_account_repo,
            ynab_account_meta_repo,
            ynab_client,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use datamize_domain::db::{
//         ynab::{MockYnabAccountMetaRepoImpl, MockYnabAccountRepoImpl},
//         DbError,
//     };
//     use fake::{Fake, Faker};
//     use mockall::predicate::eq;
//     use ynab::{AccountsDelta, MockAccountRequestsImpl};

//     use super::*;
//     use crate::error::AppError;

//     #[tokio::test]
//     async fn get_all_ynab_accounts_issue_with_db_should_not_update_saved_delta() {
//         let mut ynab_account_repo = Box::new(MockYnabAccountRepoImpl::new());
//         let mut ynab_account_meta_repo = Box::new(MockYnabAccountMetaRepoImpl::new());
//         let mut ynab_client = MockAccountRequestsImpl::new();

//         ynab_account_meta_repo
//             .expect_get_delta()
//             .once()
//             .returning(|| Err(DbError::NotFound));

//         let accounts: Vec<Account> = Faker.fake();
//         let accounts_cloned = accounts.clone();
//         ynab_client
//             .expect_get_accounts_delta()
//             .once()
//             .returning(move |_| {
//                 Ok(AccountsDelta {
//                     server_knowledge: Faker.fake(),
//                     accounts: accounts_cloned.clone(),
//                 })
//             });

//         ynab_account_repo
//             .expect_update_all()
//             .once()
//             .returning(|_| Err(DbError::BackendError(sqlx::Error::RowNotFound.to_string())));

//         ynab_account_meta_repo.expect_set_delta().never();

//         ynab_account_repo.expect_get_all().never();

//         let mut ynab_account_service = YnabAccountService {
//             ynab_account_repo,
//             ynab_account_meta_repo,
//             ynab_client: Arc::new(ynab_client),
//         };

//         let actual = ynab_account_service.get_all_ynab_accounts().await;

//         assert!(matches!(actual, Err(AppError::InternalServerError(_))));
//     }
// }
