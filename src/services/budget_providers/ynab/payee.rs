use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use ynab::{Client, Payee};

use crate::{
    db::budget_providers::ynab::{YnabPayeeMetaRepo, YnabPayeeRepo},
    error::DatamizeResult,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YnabPayeeServiceExt {
    async fn get_all_ynab_payees(&mut self) -> DatamizeResult<Vec<Payee>>;
}

pub struct YnabPayeeService<YPR: YnabPayeeRepo, YPMR: YnabPayeeMetaRepo> {
    pub ynab_payee_repo: YPR,
    pub ynab_payee_meta_repo: YPMR,
    pub ynab_client: Arc<Client>,
}

#[async_trait]
impl<YPR, YPMR> YnabPayeeServiceExt for YnabPayeeService<YPR, YPMR>
where
    YPR: YnabPayeeRepo + Sync + Send,
    YPMR: YnabPayeeMetaRepo + Sync + Send,
{
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
