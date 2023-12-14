use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    client::Response, error::YnabResult, Client, ScheduledTransactionDetail,
    ScheduledTransactionsDetailDelta,
};

#[async_trait]
pub trait ScheduledTransactionRequests {
    async fn get_scheduled_transactions(&self) -> YnabResult<Vec<ScheduledTransactionDetail>>;

    async fn get_scheduled_transactions_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<ScheduledTransactionsDetailDelta>;

    async fn get_scheduled_transaction_by_id(
        &self,
        scheduled_transaction_id: &str,
    ) -> YnabResult<ScheduledTransactionDetail>;
}

#[async_trait]
impl ScheduledTransactionRequests for Client {
    async fn get_scheduled_transactions(&self) -> YnabResult<Vec<ScheduledTransactionDetail>> {
        Ok(self
            .get_scheduled_transactions_request(None)
            .await?
            .scheduled_transactions)
    }

    async fn get_scheduled_transactions_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<ScheduledTransactionsDetailDelta> {
        self.get_scheduled_transactions_request(last_knowledge_of_server)
            .await
    }

    async fn get_scheduled_transaction_by_id(
        &self,
        scheduled_transaction_id: &str,
    ) -> YnabResult<ScheduledTransactionDetail> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            scheduled_transaction: ScheduledTransactionDetail,
        }

        let path = format!(
            "budgets/{}/scheduled_transactions/{}",
            self.get_budget_id(),
            scheduled_transaction_id
        );

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body.as_str())?;
        Ok(resp.data.scheduled_transaction)
    }
}

impl Client {
    async fn get_scheduled_transactions_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<ScheduledTransactionsDetailDelta> {
        let path = format!("budgets/{}/scheduled_transactions", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<ScheduledTransactionsDetailDelta> = Client::convert_resp(body.as_str())?;
        Ok(resp.data)
    }
}

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub ScheduledTransactionRequestsImpl {}

    impl Clone for ScheduledTransactionRequestsImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl ScheduledTransactionRequests for ScheduledTransactionRequestsImpl {
        async fn get_scheduled_transactions(&self) -> YnabResult<Vec<ScheduledTransactionDetail>>;
        async fn get_scheduled_transactions_delta(
            &self,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<ScheduledTransactionsDetailDelta>;
        async fn get_scheduled_transaction_by_id(
            &self,
            scheduled_transaction_id: &str,
        ) -> YnabResult<ScheduledTransactionDetail>;
    }
}
