use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use ynab::{Payee, PayeeRequests};

use crate::{
    db::budget_providers::ynab::{YnabPayeeMetaRepo, YnabPayeeRepo},
    error::DatamizeResult,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabPayeeServiceExt {
    async fn get_all_ynab_payees(&mut self) -> DatamizeResult<Vec<Payee>>;
}

pub struct YnabPayeeService {
    pub ynab_payee_repo: Box<dyn YnabPayeeRepo + Sync + Send>,
    pub ynab_payee_meta_repo: Box<dyn YnabPayeeMetaRepo + Sync + Send>,
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

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use mockall::{mock, predicate::eq};
    use ynab::{Payee, PayeesDelta, YnabResult};

    use super::*;
    use crate::{
        db::budget_providers::ynab::{MockYnabPayeeMetaRepo, MockYnabPayeeRepo},
        error::AppError,
    };

    mock! {
        YnabClient {}
        #[async_trait]
        impl PayeeRequests for YnabClient {
            async fn get_payees(&self) -> YnabResult<Vec<Payee>>;
            async fn get_payees_delta(
                &self,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<PayeesDelta>;
            async fn get_payee_by_id(&self, payee_id: &str) -> YnabResult<Payee>;
        }
    }

    #[tokio::test]
    async fn get_all_ynab_payees_success() {
        let mut ynab_payee_repo = Box::new(MockYnabPayeeRepo::new());
        let mut ynab_payee_meta_repo = Box::new(MockYnabPayeeMetaRepo::new());
        let mut ynab_client = MockYnabClient::new();

        ynab_payee_meta_repo
            .expect_get_delta()
            .once()
            .returning(|| Err(AppError::ResourceNotFound));

        let payees = vec![
            Payee {
                id: Faker.fake(),
                name: Faker.fake(),
                transfer_account_id: Faker.fake(),
                deleted: false,
            },
            Payee {
                id: Faker.fake(),
                name: Faker.fake(),
                transfer_account_id: Faker.fake(),
                deleted: false,
            },
        ];
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
        let mut ynab_payee_repo = Box::new(MockYnabPayeeRepo::new());
        let mut ynab_payee_meta_repo = Box::new(MockYnabPayeeMetaRepo::new());
        let mut ynab_client = MockYnabClient::new();

        ynab_payee_meta_repo
            .expect_get_delta()
            .once()
            .returning(|| Err(AppError::ResourceNotFound));

        let payees = vec![
            Payee {
                id: Faker.fake(),
                name: Faker.fake(),
                transfer_account_id: Faker.fake(),
                deleted: false,
            },
            Payee {
                id: Faker.fake(),
                name: Faker.fake(),
                transfer_account_id: Faker.fake(),
                deleted: false,
            },
        ];
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

        ynab_payee_repo.expect_update_all().once().returning(|_| {
            Err(AppError::InternalServerError(
                sqlx::Error::RowNotFound.into(),
            ))
        });

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
