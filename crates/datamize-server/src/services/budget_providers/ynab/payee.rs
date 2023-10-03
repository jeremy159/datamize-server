use std::sync::Arc;

use anyhow::Context;
use datamize_domain::{
    async_trait,
    db::ynab::{DynYnabPayeeMetaRepo, DynYnabPayeeRepo},
};
use dyn_clone::{clone_trait_object, DynClone};
use ynab::{Payee, PayeeRequests};

use crate::error::DatamizeResult;

#[async_trait]
pub trait YnabPayeeServiceExt: DynClone + Send + Sync {
    async fn get_all_ynab_payees(&mut self) -> DatamizeResult<Vec<Payee>>;
}

clone_trait_object!(YnabPayeeServiceExt);

pub type DynYnabPayeeService = Box<dyn YnabPayeeServiceExt>;

#[cfg(test)]
mockall::mock! {
    pub YnabPayeeService {}

    impl Clone for YnabPayeeService {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl YnabPayeeServiceExt for YnabPayeeService {
        async fn get_all_ynab_payees(&mut self) -> DatamizeResult<Vec<Payee>>;
    }
}

#[derive(Clone)]
pub struct YnabPayeeService {
    pub ynab_payee_repo: DynYnabPayeeRepo,
    pub ynab_payee_meta_repo: DynYnabPayeeMetaRepo,
    pub ynab_client: Arc<dyn PayeeRequests + Send + Sync>,
}

#[async_trait]
impl YnabPayeeServiceExt for YnabPayeeService {
    #[tracing::instrument(skip(self))]
    async fn get_all_ynab_payees(&mut self) -> DatamizeResult<Vec<Payee>> {
        let saved_payees_delta = self.ynab_payee_meta_repo.get_delta().await.ok();

        let payees_delta = self
            .ynab_client
            .get_payees_delta(saved_payees_delta)
            .await
            .context("failed to get payees from ynab's API")?;

        let payees = payees_delta
            .payees
            .into_iter()
            .filter(|a| !a.deleted)
            .collect::<Vec<_>>();

        self.ynab_payee_repo
            .update_all(&payees)
            .await
            .context("failed to save payees in database")?;

        self.ynab_payee_meta_repo
            .set_delta(payees_delta.server_knowledge)
            .await
            .context("failed to save last known server knowledge of payees in redis")?;

        let saved_payees = self
            .ynab_payee_repo
            .get_all()
            .await
            .context("failed to get payees from database")?;

        Ok(saved_payees)
    }
}

impl YnabPayeeService {
    pub fn new_boxed(
        ynab_payee_repo: DynYnabPayeeRepo,
        ynab_payee_meta_repo: DynYnabPayeeMetaRepo,
        ynab_client: Arc<dyn PayeeRequests + Send + Sync>,
    ) -> Box<Self> {
        Box::new(Self {
            ynab_payee_repo,
            ynab_payee_meta_repo,
            ynab_client,
        })
    }
}

#[cfg(test)]
mod tests {
    use datamize_domain::db::{
        ynab::{MockYnabPayeeMetaRepoImpl, MockYnabPayeeRepoImpl},
        DbError,
    };
    use fake::{Fake, Faker};
    use mockall::predicate::eq;
    use ynab::{MockPayeeRequests, Payee, PayeesDelta};

    use super::*;
    use crate::error::AppError;

    // FIXME: Test sometimes failling with `panicked at 'MockYnabPayeeRepoImpl::update_all(?): No matching expectation found'`
    // #[tokio::test]
    async fn get_all_ynab_payees_success() {
        let mut ynab_payee_repo = Box::new(MockYnabPayeeRepoImpl::new());
        let mut ynab_payee_meta_repo = Box::new(MockYnabPayeeMetaRepoImpl::new());
        let mut ynab_client = MockPayeeRequests::new();

        ynab_payee_meta_repo
            .expect_get_delta()
            .once()
            .returning(|| Err(DbError::NotFound));

        let payees: Vec<Payee> = Faker.fake();
        let payees_cloned = payees.clone();
        ynab_client
            .expect_get_payees_delta()
            .once()
            .returning(move |_| {
                Ok(PayeesDelta {
                    server_knowledge: Faker.fake(),
                    payees: payees_cloned.clone(),
                })
            });

        ynab_payee_repo
            .expect_update_all()
            .once()
            .with(eq(payees.clone()))
            .returning(|_| Ok(()));

        ynab_payee_meta_repo
            .expect_set_delta()
            .once()
            .returning(|_| Ok(()));

        ynab_payee_repo
            .expect_get_all()
            .once()
            .returning(move || Ok(payees.clone()));

        let mut ynab_payee_service = YnabPayeeService {
            ynab_payee_repo,
            ynab_payee_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        ynab_payee_service.get_all_ynab_payees().await.unwrap();
    }

    #[tokio::test]
    async fn get_all_ynab_payees_issue_with_db_should_not_update_saved_delta() {
        let mut ynab_payee_repo = Box::new(MockYnabPayeeRepoImpl::new());
        let mut ynab_payee_meta_repo = Box::new(MockYnabPayeeMetaRepoImpl::new());
        let mut ynab_client = MockPayeeRequests::new();

        ynab_payee_meta_repo
            .expect_get_delta()
            .once()
            .returning(|| Err(DbError::NotFound));

        let payees: Vec<Payee> = Faker.fake();
        let payees_cloned = payees.clone();
        ynab_client
            .expect_get_payees_delta()
            .once()
            .returning(move |_| {
                Ok(PayeesDelta {
                    server_knowledge: Faker.fake(),
                    payees: payees_cloned.clone(),
                })
            });

        ynab_payee_repo
            .expect_update_all()
            .once()
            .returning(|_| Err(DbError::BackendError(sqlx::Error::RowNotFound.to_string())));

        ynab_payee_meta_repo.expect_set_delta().never();

        ynab_payee_repo.expect_get_all().never();

        let mut ynab_payee_service = YnabPayeeService {
            ynab_payee_repo,
            ynab_payee_meta_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = ynab_payee_service.get_all_ynab_payees().await;

        assert!(matches!(actual, Err(AppError::InternalServerError(_))));
    }
}
