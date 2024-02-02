use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    client::Response, error::YnabResult, Client, SaveTransaction, TransactionDetail,
    TransactionType, TransactionsDetailDelta, TransactionsParentPath, TransactionsRequestQuery,
    UpdateTransaction,
};

#[async_trait]
pub trait TransactionRequests {
    async fn get_transactions(&self) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn get_transactions_since(&self, since_date: &str) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_since_delta(
        &self,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn get_transactions_of(
        &self,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_of_delta(
        &self,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn get_transactions_since_date_of(
        &self,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_since_date_of_delta(
        &self,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn get_transactions_by_account_id(
        &self,
        account_id: &str,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_by_account_id_delta(
        &self,
        account_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn get_transactions_by_account_id_since(
        &self,
        account_id: &str,
        since_date: &str,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_by_account_id_since_delta(
        &self,
        account_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn get_transactions_by_account_id_of(
        &self,
        account_id: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_by_account_id_of_delta(
        &self,
        account_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn get_transactions_by_account_id_since_date_of(
        &self,
        account_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn get_transactions_by_account_id_since_date_of_delta(
        &self,
        account_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta>;

    async fn create_transaction(&self, data: SaveTransaction) -> YnabResult<TransactionDetail>;

    async fn create_transactions(
        &self,
        data: Vec<SaveTransaction>,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn update_transactions(
        &self,
        data: Vec<UpdateTransaction>,
    ) -> YnabResult<Vec<TransactionDetail>>;

    async fn import_transactions(&self) -> YnabResult<Vec<String>>;

    async fn get_transaction_by_id(&self, transaction_id: &str) -> YnabResult<TransactionDetail>;

    async fn update_transaction(
        &self,
        transaction_id: &str,
        data: SaveTransaction,
    ) -> YnabResult<TransactionDetail>;
}

#[async_trait]
impl TransactionRequests for Client {
    async fn get_transactions(&self) -> YnabResult<Vec<TransactionDetail>> {
        Ok(self
            .get_transactions_request(None, None)
            .await?
            .transactions)
    }

    async fn get_transactions_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_transactions_request(None, Some(query)).await
    }

    async fn get_transactions_since(&self, since_date: &str) -> YnabResult<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_transactions_request(None, Some(query))
            .await?
            .transactions)
    }

    async fn get_transactions_since_delta(
        &self,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_transactions_request(None, Some(query)).await
    }

    async fn get_transactions_of(
        &self,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(None, Some(query))
            .await?
            .transactions)
    }

    async fn get_transactions_of_delta(
        &self,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(None, Some(query)).await
    }

    async fn get_transactions_since_date_of(
        &self,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(None, Some(query))
            .await?
            .transactions)
    }

    async fn get_transactions_since_date_of_delta(
        &self,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(None, Some(query)).await
    }

    async fn get_transactions_by_account_id(
        &self,
        account_id: &str,
    ) -> YnabResult<Vec<TransactionDetail>> {
        Ok(self
            .get_transactions_request(Some(TransactionsParentPath::Accounts(account_id)), None)
            .await?
            .transactions)
    }

    async fn get_transactions_by_account_id_delta(
        &self,
        account_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_transactions_request(
            Some(TransactionsParentPath::Accounts(account_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_account_id_since(
        &self,
        account_id: &str,
        since_date: &str,
    ) -> YnabResult<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Accounts(account_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_account_id_since_delta(
        &self,
        account_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_transactions_request(
            Some(TransactionsParentPath::Accounts(account_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_account_id_of(
        &self,
        account_id: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Accounts(account_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_account_id_of_delta(
        &self,
        account_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(
            Some(TransactionsParentPath::Accounts(account_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_account_id_since_date_of(
        &self,
        account_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Accounts(account_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_account_id_since_date_of_delta(
        &self,
        account_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(
            Some(TransactionsParentPath::Accounts(account_id)),
            Some(query),
        )
        .await
    }

    async fn create_transaction(&self, data: SaveTransaction) -> YnabResult<TransactionDetail> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Body {
            transaction: SaveTransaction,
        }
        let body: Body = Body { transaction: data };

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            transaction_ids: Vec<String>,
            transaction: TransactionDetail,
            duplicate_import_ids: Vec<String>,
        }

        let path = format!("budgets/{}/transactions", self.get_budget_id());

        let body_resp = self.post(&path, Some(&body)).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.transaction)
    }

    async fn create_transactions(
        &self,
        data: Vec<SaveTransaction>,
    ) -> YnabResult<Vec<TransactionDetail>> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Body {
            transactions: Vec<SaveTransaction>,
        }
        let body: Body = Body { transactions: data };

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            transaction_ids: Vec<String>,
            transactions: Vec<TransactionDetail>,
            duplicate_import_ids: Vec<String>,
        }

        let path = format!("budgets/{}/transactions", self.get_budget_id());

        let body_resp = self.post(&path, Some(&body)).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.transactions)
    }

    async fn update_transactions(
        &self,
        data: Vec<UpdateTransaction>,
    ) -> YnabResult<Vec<TransactionDetail>> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Body {
            transactions: Vec<UpdateTransaction>,
        }
        let body: Body = Body { transactions: data };

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            transaction_ids: Vec<String>,
            transactions: Vec<TransactionDetail>,
            duplicate_import_ids: Vec<String>,
        }

        let path = format!("budgets/{}/transactions", self.get_budget_id());

        let body_resp = self.patch(&path, &body).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.transactions)
    }

    async fn import_transactions(&self) -> YnabResult<Vec<String>> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            transaction_ids: Vec<String>,
        }

        let path = format!("budgets/{}/transactions/import", self.get_budget_id());

        let body_resp = self
            .post::<String>(&path, None)
            .send()
            .await?
            .text()
            .await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.transaction_ids)
    }

    async fn get_transaction_by_id(&self, transaction_id: &str) -> YnabResult<TransactionDetail> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            transaction: TransactionDetail,
        }

        let path = format!(
            "budgets/{}/transactions/{}",
            self.get_budget_id(),
            transaction_id
        );

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.transaction)
    }

    async fn update_transaction(
        &self,
        transaction_id: &str,
        data: SaveTransaction,
    ) -> YnabResult<TransactionDetail> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Body {
            transaction: SaveTransaction,
        }
        let body: Body = Body { transaction: data };

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            transaction: TransactionDetail,
        }

        let path = format!(
            "budgets/{}/transactions/{}",
            self.get_budget_id(),
            transaction_id
        );

        let body_resp = self.put(&path, &body).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.transaction)
    }
}

impl Client {
    async fn get_transactions_request(
        &self,
        parent_path: Option<TransactionsParentPath<&str>>,
        query: Option<TransactionsRequestQuery>,
    ) -> YnabResult<TransactionsDetailDelta> {
        let path = match parent_path {
            Some(tp) => match tp {
                TransactionsParentPath::Accounts(id) => format!(
                    "budgets/{}/accounts/{}/transactions",
                    self.get_budget_id(),
                    id
                ),
                TransactionsParentPath::Categories(id) => format!(
                    "budgets/{}/categories/{}/transactions",
                    self.get_budget_id(),
                    id
                ),
                TransactionsParentPath::Payees(id) => format!(
                    "budgets/{}/payees/{}/transactions",
                    self.get_budget_id(),
                    id
                ),
            },
            None => format!("budgets/{}/transactions", self.get_budget_id()),
        };

        let request_builder = if query.is_some() {
            self.get_with_query(&path, &query)
        } else {
            self.get(&path)
        };

        let body = request_builder.send().await?.text().await?;

        let resp: Response<TransactionsDetailDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }
}

#[cfg(any(feature = "testutils", test))]
mockall::mock! {
    pub TransactionRequestsImpl {}

    impl Clone for TransactionRequestsImpl {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl TransactionRequests for TransactionRequestsImpl {
        async fn get_transactions(&self) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_delta(
            &self,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn get_transactions_since(&self, since_date: &str) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_since_delta(
            &self,
            since_date: &str,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn get_transactions_of(
            &self,
            transaction_type: TransactionType,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_of_delta(
            &self,
            transaction_type: TransactionType,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn get_transactions_since_date_of(
            &self,
            since_date: &str,
            transaction_type: TransactionType,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_since_date_of_delta(
            &self,
            since_date: &str,
            transaction_type: TransactionType,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn get_transactions_by_account_id(
            &self,
            account_id: &str,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_by_account_id_delta(
            &self,
            account_id: &str,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn get_transactions_by_account_id_since(
            &self,
            account_id: &str,
            since_date: &str,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_by_account_id_since_delta(
            &self,
            account_id: &str,
            since_date: &str,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn get_transactions_by_account_id_of(
            &self,
            account_id: &str,
            transaction_type: TransactionType,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_by_account_id_of_delta(
            &self,
            account_id: &str,
            transaction_type: TransactionType,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn get_transactions_by_account_id_since_date_of(
            &self,
            account_id: &str,
            since_date: &str,
            transaction_type: TransactionType,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn get_transactions_by_account_id_since_date_of_delta(
            &self,
            account_id: &str,
            since_date: &str,
            transaction_type: TransactionType,
            last_knowledge_of_server: Option<i64>,
        ) -> YnabResult<TransactionsDetailDelta>;
        async fn create_transaction(&self, data: SaveTransaction) -> YnabResult<TransactionDetail>;
        async fn create_transactions(
            &self,
            data: Vec<SaveTransaction>,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn update_transactions(
            &self,
            data: Vec<UpdateTransaction>,
        ) -> YnabResult<Vec<TransactionDetail>>;
        async fn import_transactions(&self) -> YnabResult<Vec<String>>;
        async fn get_transaction_by_id(&self, transaction_id: &str) -> YnabResult<TransactionDetail>;
        async fn update_transaction(
            &self,
            transaction_id: &str,
            data: SaveTransaction,
        ) -> YnabResult<TransactionDetail>;
    }
}
