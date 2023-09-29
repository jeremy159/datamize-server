use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{client::Response, error::YnabResult, Client, Payee, PayeesDelta};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PayeeRequests {
    async fn get_payees(&self) -> YnabResult<Vec<Payee>>;

    async fn get_payees_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<PayeesDelta>;

    async fn get_payee_by_id(&self, payee_id: &str) -> YnabResult<Payee>;
}

#[async_trait]
impl PayeeRequests for Client {
    async fn get_payees(&self) -> YnabResult<Vec<Payee>> {
        Ok(self.get_payees_request(None).await?.payees)
    }

    async fn get_payees_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<PayeesDelta> {
        self.get_payees_request(last_knowledge_of_server).await
    }

    async fn get_payee_by_id(&self, payee_id: &str) -> YnabResult<Payee> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            payee: Payee,
        }

        let path = format!("budgets/{}/payees/{}", self.get_budget_id(), payee_id);

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.payee)
    }
}

impl Client {
    async fn get_payees_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<PayeesDelta> {
        let path = format!("budgets/{}/payees", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<PayeesDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }
}
