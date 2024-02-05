use std::sync::Arc;

use anyhow::Context;
use datamize_domain::{
    async_trait,
    db::ynab::{DynYnabPayeeMetaRepo, DynYnabPayeeRepo},
};
use ynab::{Payee, PayeeRequests};

use crate::error::DatamizeResult;

#[async_trait]
pub trait YnabPayeeServiceExt: Send + Sync {
    async fn get_all_ynab_payees(&self) -> DatamizeResult<Vec<Payee>>;
}

pub type DynYnabPayeeService = Arc<dyn YnabPayeeServiceExt>;

#[derive(Clone)]
pub struct YnabPayeeService {
    pub ynab_payee_repo: DynYnabPayeeRepo,
    pub ynab_payee_meta_repo: DynYnabPayeeMetaRepo,
    pub ynab_client: Arc<dyn PayeeRequests + Send + Sync>,
}

#[async_trait]
impl YnabPayeeServiceExt for YnabPayeeService {
    #[tracing::instrument(skip(self))]
    async fn get_all_ynab_payees(&self) -> DatamizeResult<Vec<Payee>> {
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
    pub fn new_arced(
        ynab_payee_repo: DynYnabPayeeRepo,
        ynab_payee_meta_repo: DynYnabPayeeMetaRepo,
        ynab_client: Arc<dyn PayeeRequests + Send + Sync>,
    ) -> Arc<Self> {
        Arc::new(Self {
            ynab_payee_repo,
            ynab_payee_meta_repo,
            ynab_client,
        })
    }
}
