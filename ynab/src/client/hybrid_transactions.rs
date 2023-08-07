use async_trait::async_trait;

use crate::{
    client::Response, error::YnabResult, Client, HybridTransaction, HybridTransationsDelta,
    TransactionType, TransactionsParentPath, TransactionsRequestQuery,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HybridTransactionRequests {
    async fn get_transactions_by_category_id(
        &self,
        category_id: &str,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_category_id_delta(
        &self,
        category_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;

    async fn get_transactions_by_category_id_since(
        &self,
        category_id: &str,
        since_date: &str,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_category_id_since_delta(
        &self,
        category_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;

    async fn get_transactions_by_category_id_of(
        &self,
        category_id: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_category_id_of_delta(
        &self,
        category_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;

    async fn get_transactions_by_category_id_since_date_of(
        &self,
        category_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_category_id_since_date_of_delta(
        &self,
        category_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;

    async fn get_transactions_by_payee_id(
        &self,
        payee_id: &str,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_payee_id_delta(
        &self,
        payee_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;

    async fn get_transactions_by_payee_id_since(
        &self,
        payee_id: &str,
        since_date: &str,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_payee_id_since_delta(
        &self,
        payee_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;

    async fn get_transactions_by_payee_id_of(
        &self,
        payee_id: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_payee_id_of_delta(
        &self,
        payee_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;

    async fn get_transactions_by_payee_id_since_date_of(
        &self,
        payee_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>>;

    async fn get_transactions_by_payee_id_since_date_of_delta(
        &self,
        payee_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta>;
}

#[async_trait]
impl HybridTransactionRequests for Client {
    async fn get_transactions_by_category_id(
        &self,
        category_id: &str,
    ) -> YnabResult<Vec<HybridTransaction>> {
        Ok(self
            .get_hybrid_transactions_request(
                Some(TransactionsParentPath::Categories(category_id)),
                None,
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_category_id_delta(
        &self,
        category_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_category_id_since(
        &self,
        category_id: &str,
        since_date: &str,
    ) -> YnabResult<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_hybrid_transactions_request(
                Some(TransactionsParentPath::Categories(category_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_category_id_since_delta(
        &self,
        category_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_category_id_of(
        &self,
        category_id: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_hybrid_transactions_request(
                Some(TransactionsParentPath::Categories(category_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_category_id_of_delta(
        &self,
        category_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_category_id_since_date_of(
        &self,
        category_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        Ok(self
            .get_hybrid_transactions_request(
                Some(TransactionsParentPath::Categories(category_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_category_id_since_date_of_delta(
        &self,
        category_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_payee_id(
        &self,
        payee_id: &str,
    ) -> YnabResult<Vec<HybridTransaction>> {
        Ok(self
            .get_hybrid_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), None)
            .await?
            .transactions)
    }

    async fn get_transactions_by_payee_id_delta(
        &self,
        payee_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Payees(payee_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_payee_id_since(
        &self,
        payee_id: &str,
        since_date: &str,
    ) -> YnabResult<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_hybrid_transactions_request(
                Some(TransactionsParentPath::Payees(payee_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_payee_id_since_delta(
        &self,
        payee_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Payees(payee_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_payee_id_of(
        &self,
        payee_id: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_hybrid_transactions_request(
                Some(TransactionsParentPath::Payees(payee_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_payee_id_of_delta(
        &self,
        payee_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Payees(payee_id)),
            Some(query),
        )
        .await
    }

    async fn get_transactions_by_payee_id_since_date_of(
        &self,
        payee_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> YnabResult<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        Ok(self
            .get_hybrid_transactions_request(
                Some(TransactionsParentPath::Payees(payee_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    async fn get_transactions_by_payee_id_since_date_of_delta(
        &self,
        payee_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<HybridTransationsDelta> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        self.get_hybrid_transactions_request(
            Some(TransactionsParentPath::Payees(payee_id)),
            Some(query),
        )
        .await
    }
}

impl Client {
    async fn get_hybrid_transactions_request(
        &self,
        parent_path: Option<TransactionsParentPath<&str>>,
        query: Option<TransactionsRequestQuery>,
    ) -> YnabResult<HybridTransationsDelta> {
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

        let resp: Response<HybridTransationsDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }
}
