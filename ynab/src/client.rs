use super::Result;
use crate::error::{ApiErrorResponse, Error};
use crate::types::{
    Account, AccountsDelta, BaseBudgetSumary, BaseTransactionDetail, BudgetDetail,
    BudgetDetailDelta, BudgetSettings, BudgetSummary, BudgetSummaryWithAccounts, Category,
    CategoryGroupWithCategories, CategoryGroupWithCategoriesDelta, HybridTransaction, MonthDetail,
    MonthSummary, MonthSummaryDelta, Payee, PayeeLocation, PayeesDelta, SaveAccount,
    SaveMonthCategory, SaveTransaction, ScheduledTransactionDetail,
    ScheduledTransactionsDetailDelta, TransactionDetail, TransactionType, TransactionsParentPath,
    TransactionsRequestQuery, TransationsDetailDelta, UpdateTransaction,
};
use reqwest::{header, Client as ReqwestClient, RequestBuilder, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response<T> {
    pub data: T,
}

#[derive(Debug)]
pub struct Client {
    ynab_api_token: String,
    ynab_base_url: Url,
    pub default_budget_id: Option<String>,
    http_client: ReqwestClient,
}

impl Client {
    pub fn new(ynab_api_token: &str, ynab_base_url: &str) -> Result<Self> {
        let ynab_base_url = Url::parse(ynab_base_url)
            .unwrap_or_else(|_| panic!("`{}` to be a valid URL", ynab_base_url));
        let http_client = Client::build_http_client()?;

        Ok(Self {
            ynab_api_token: ynab_api_token.into(),
            ynab_base_url,
            http_client,
            default_budget_id: None,
        })
    }

    pub fn new_with_default_budget_id(
        ynab_api_token: &str,
        ynab_base_url: &str,
        default_budget_id: &str,
    ) -> Result<Self> {
        let ynab_base_url = Url::parse(ynab_base_url)
            .unwrap_or_else(|_| panic!("`{}` to be a valid URL", ynab_base_url));
        let http_client = Client::build_http_client()?;

        Ok(Self {
            ynab_api_token: ynab_api_token.into(),
            ynab_base_url,
            http_client,
            default_budget_id: Some(default_budget_id.into()),
        })
    }

    pub fn set_default_budget_id(&mut self, default_budget_id: &str) -> &Self {
        self.default_budget_id = Some(default_budget_id.into());

        self
    }

    async fn get_budgets_request<T>(&self, with_accounts: bool) -> Result<Vec<T>>
    where
        T: AsRef<BaseBudgetSumary> + DeserializeOwned,
    {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner<I> {
            budgets: Vec<I>,
            default_budget: Option<I>,
        }

        let body = match with_accounts {
            true => self.get_with_query("budgets", &[("include_accounts", "true")]),
            false => self.get("budgets"),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<Inner<T>> = Client::convert_resp(body)?;
        Ok(resp.data.budgets)
    }

    pub async fn get_budgets(&self) -> Result<Vec<BudgetSummary>> {
        self.get_budgets_request(false).await
    }

    pub async fn get_budgets_with_accounts(&self) -> Result<Vec<BudgetSummaryWithAccounts>> {
        self.get_budgets_request(true).await
    }

    async fn get_budget_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<BudgetDetailDelta> {
        let path = format!("budgets/{}", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<BudgetDetailDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }

    pub async fn get_budget(&self) -> Result<BudgetDetail> {
        Ok(self.get_budget_request(None).await?.budget)
    }

    pub async fn get_budget_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<BudgetDetailDelta> {
        self.get_budget_request(last_knowledge_of_server).await
    }

    pub async fn get_budget_settings(&self) -> Result<BudgetSettings> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            settings: BudgetSettings,
        }

        let path = format!("budgets/{}/settings", self.get_budget_id());

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.settings)
    }

    async fn get_accounts_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<AccountsDelta> {
        let path = format!("budgets/{}/accounts", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<AccountsDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }

    pub async fn get_accounts(&self) -> Result<Vec<Account>> {
        Ok(self.get_accounts_request(None).await?.accounts)
    }

    pub async fn get_accounts_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<AccountsDelta> {
        self.get_accounts_request(last_knowledge_of_server).await
    }

    pub async fn create_account(&self, data: SaveAccount) -> Result<Account> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Body {
            account: SaveAccount,
        }
        let body: Body = Body { account: data };

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            account: Account,
        }

        let path = format!("budgets/{}/accounts", self.get_budget_id());

        let body_resp = self.post(&path, Some(&body)).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.account)
    }

    pub async fn get_account_by_id(&self, account_id: &str) -> Result<Account> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            account: Account,
        }

        let path = format!("budgets/{}/accounts/{}", self.get_budget_id(), account_id);

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.account)
    }

    async fn get_categories_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<CategoryGroupWithCategoriesDelta> {
        let path = format!("budgets/{}/categories", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<CategoryGroupWithCategoriesDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }

    pub async fn get_categories(&self) -> Result<Vec<CategoryGroupWithCategories>> {
        Ok(self.get_categories_request(None).await?.category_groups)
    }

    pub async fn get_categories_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<CategoryGroupWithCategoriesDelta> {
        self.get_categories_request(last_knowledge_of_server).await
    }

    async fn get_category_by_id_request(
        &self,
        category_id: &str,
        month: Option<&str>,
    ) -> Result<Category> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            category: Category,
        }

        let path = match month {
            Some(m) => format!(
                "budgets/{}/months/{}/categories/{}",
                self.get_budget_id(),
                m,
                category_id
            ),
            None => format!(
                "budgets/{}/categories/{}",
                self.get_budget_id(),
                category_id
            ),
        };

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.category)
    }

    pub async fn get_category_by_id(&self, category_id: &str) -> Result<Category> {
        self.get_category_by_id_request(category_id, None).await
    }

    pub async fn get_category_by_id_for(&self, category_id: &str, month: &str) -> Result<Category> {
        self.get_category_by_id_request(category_id, Some(month))
            .await
    }

    pub async fn update_category_for(
        &self,
        category_id: &str,
        month: &str,
        data: SaveMonthCategory,
    ) -> Result<Category> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Body {
            category: SaveMonthCategory,
        }
        let body: Body = Body { category: data };

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            category: Category,
        }

        let path = format!(
            "budgets/{}/months/{}/categories/{}",
            self.get_budget_id(),
            month,
            category_id
        );

        let body_resp = self.patch(&path, &body).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body_resp)?;
        Ok(resp.data.category)
    }

    async fn get_payees_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<PayeesDelta> {
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

    pub async fn get_payees(&self) -> Result<Vec<Payee>> {
        Ok(self.get_payees_request(None).await?.payees)
    }

    pub async fn get_payees_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<PayeesDelta> {
        self.get_payees_request(last_knowledge_of_server).await
    }

    pub async fn get_payee_by_id(&self, payee_id: &str) -> Result<Payee> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            payee: Payee,
        }

        let path = format!("budgets/{}/payees/{}", self.get_budget_id(), payee_id);

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.payee)
    }

    async fn get_payee_locations_request(
        &self,
        payee_id: Option<&str>,
    ) -> Result<Vec<PayeeLocation>> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            payee_locations: Vec<PayeeLocation>,
        }

        let path = match payee_id {
            Some(p) => format!(
                "budgets/{}/payees/{}/payee_locations",
                self.get_budget_id(),
                p
            ),
            None => format!("budgets/{}/payee_locations", self.get_budget_id()),
        };

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.payee_locations)
    }

    pub async fn get_payee_locations(&self) -> Result<Vec<PayeeLocation>> {
        self.get_payee_locations_request(None).await
    }

    pub async fn get_payee_locations_for(&self, payee_id: &str) -> Result<Vec<PayeeLocation>> {
        self.get_payee_locations_request(Some(payee_id)).await
    }

    pub async fn get_payee_location_by_id(&self, payee_location_id: &str) -> Result<PayeeLocation> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            payee_location: PayeeLocation,
        }

        let path = format!(
            "budgets/{}/payee_locations/{}",
            self.get_budget_id(),
            payee_location_id
        );

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.payee_location)
    }

    async fn get_months_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<MonthSummaryDelta> {
        let path = format!("budgets/{}/months", self.get_budget_id());

        let body = match last_knowledge_of_server {
            Some(k) => self.get_with_query(&path, &[("last_knowledge_of_server", k)]),
            None => self.get(&path),
        }
        .send()
        .await?
        .text()
        .await?;

        let resp: Response<MonthSummaryDelta> = Client::convert_resp(body)?;
        Ok(resp.data)
    }

    pub async fn get_months(&self) -> Result<Vec<MonthSummary>> {
        Ok(self.get_months_request(None).await?.months)
    }

    pub async fn get_months_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<MonthSummaryDelta> {
        self.get_months_request(last_knowledge_of_server).await
    }

    pub async fn get_month_by_date(&self, date: &str) -> Result<MonthDetail> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct Inner {
            month: MonthDetail,
        }

        let path = format!("budgets/{}/months/{}", self.get_budget_id(), date);

        let body = self.get(&path).send().await?.text().await?;

        let resp: Response<Inner> = Client::convert_resp(body)?;
        Ok(resp.data.month)
    }

    async fn get_transactions_request<T>(
        &self,
        parent_path: Option<TransactionsParentPath<&str>>,
        query: Option<TransactionsRequestQuery>,
    ) -> Result<TransationsDetailDelta<T>>
    where
        T: AsRef<BaseTransactionDetail> + DeserializeOwned,
    {
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

        let resp: Response<TransationsDetailDelta<T>> = Client::convert_resp(body)?;
        Ok(resp.data)
    }

    pub async fn get_transactions(&self) -> Result<Vec<TransactionDetail>> {
        Ok(self
            .get_transactions_request(None, None)
            .await?
            .transactions)
    }

    pub async fn get_transactions_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_transactions_request(None, Some(query)).await
    }

    pub async fn get_transactions_since(&self, since_date: &str) -> Result<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_transactions_request(None, Some(query))
            .await?
            .transactions)
    }

    pub async fn get_transactions_since_delta(
        &self,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_transactions_request(None, Some(query)).await
    }

    pub async fn get_transactions_of(
        &self,
        transaction_type: TransactionType,
    ) -> Result<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(None, Some(query))
            .await?
            .transactions)
    }

    pub async fn get_transactions_of_delta(
        &self,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(None, Some(query)).await
    }

    pub async fn get_transactions_since_date_of(
        &self,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> Result<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(None, Some(query))
            .await?
            .transactions)
    }

    pub async fn get_transactions_since_date_of_delta(
        &self,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(None, Some(query)).await
    }

    pub async fn get_transactions_by_account_id(
        &self,
        account_id: &str,
    ) -> Result<Vec<TransactionDetail>> {
        Ok(self
            .get_transactions_request(Some(TransactionsParentPath::Accounts(account_id)), None)
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_account_id_delta(
        &self,
        account_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_transactions_request(
            Some(TransactionsParentPath::Accounts(account_id)),
            Some(query),
        )
        .await
    }

    pub async fn get_transactions_by_account_id_since(
        &self,
        account_id: &str,
        since_date: &str,
    ) -> Result<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Accounts(account_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_account_id_since_delta(
        &self,
        account_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_transactions_request(
            Some(TransactionsParentPath::Accounts(account_id)),
            Some(query),
        )
        .await
    }

    pub async fn get_transactions_by_account_id_of(
        &self,
        account_id: &str,
        transaction_type: TransactionType,
    ) -> Result<Vec<TransactionDetail>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Accounts(account_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_account_id_of_delta(
        &self,
        account_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(
            Some(TransactionsParentPath::Accounts(account_id)),
            Some(query),
        )
        .await
    }

    pub async fn get_transactions_by_account_id_since_date_of(
        &self,
        account_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> Result<Vec<TransactionDetail>> {
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

    pub async fn get_transactions_by_account_id_since_date_of_delta(
        &self,
        account_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<TransactionDetail>> {
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

    pub async fn get_transactions_by_category_id(
        &self,
        category_id: &str,
    ) -> Result<Vec<HybridTransaction>> {
        Ok(self
            .get_transactions_request(Some(TransactionsParentPath::Categories(category_id)), None)
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_category_id_delta(
        &self,
        category_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    pub async fn get_transactions_by_category_id_since(
        &self,
        category_id: &str,
        since_date: &str,
    ) -> Result<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Categories(category_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_category_id_since_delta(
        &self,
        category_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    pub async fn get_transactions_by_category_id_of(
        &self,
        category_id: &str,
        transaction_type: TransactionType,
    ) -> Result<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Categories(category_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_category_id_of_delta(
        &self,
        category_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    pub async fn get_transactions_by_category_id_since_date_of(
        &self,
        category_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> Result<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(
                Some(TransactionsParentPath::Categories(category_id)),
                Some(query),
            )
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_category_id_since_date_of_delta(
        &self,
        category_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(
            Some(TransactionsParentPath::Categories(category_id)),
            Some(query),
        )
        .await
    }

    pub async fn get_transactions_by_payee_id(
        &self,
        payee_id: &str,
    ) -> Result<Vec<HybridTransaction>> {
        Ok(self
            .get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), None)
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_payee_id_delta(
        &self,
        payee_id: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query =
            TransactionsRequestQuery::default().with_last_knowledge(last_knowledge_of_server);
        self.get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), Some(query))
            .await
    }

    pub async fn get_transactions_by_payee_id_since(
        &self,
        payee_id: &str,
        since_date: &str,
    ) -> Result<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_date(since_date);
        Ok(self
            .get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), Some(query))
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_payee_id_since_delta(
        &self,
        payee_id: &str,
        since_date: &str,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date);
        self.get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), Some(query))
            .await
    }

    pub async fn get_transactions_by_payee_id_of(
        &self,
        payee_id: &str,
        transaction_type: TransactionType,
    ) -> Result<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default().with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), Some(query))
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_payee_id_of_delta(
        &self,
        payee_id: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), Some(query))
            .await
    }

    pub async fn get_transactions_by_payee_id_since_date_of(
        &self,
        payee_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
    ) -> Result<Vec<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        Ok(self
            .get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), Some(query))
            .await?
            .transactions)
    }

    pub async fn get_transactions_by_payee_id_since_date_of_delta(
        &self,
        payee_id: &str,
        since_date: &str,
        transaction_type: TransactionType,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<TransationsDetailDelta<HybridTransaction>> {
        let query = TransactionsRequestQuery::default()
            .with_last_knowledge(last_knowledge_of_server)
            .with_date(since_date)
            .with_transaction_type(transaction_type);
        self.get_transactions_request(Some(TransactionsParentPath::Payees(payee_id)), Some(query))
            .await
    }

    pub async fn create_transaction(&self, data: SaveTransaction) -> Result<TransactionDetail> {
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

    pub async fn create_transactions(
        &self,
        data: Vec<SaveTransaction>,
    ) -> Result<Vec<TransactionDetail>> {
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

    pub async fn update_transactions(
        &self,
        data: Vec<UpdateTransaction>,
    ) -> Result<Vec<TransactionDetail>> {
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

    pub async fn import_transactions(&self) -> Result<Vec<String>> {
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

    pub async fn get_transaction_by_id(&self, transaction_id: &str) -> Result<TransactionDetail> {
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

    pub async fn update_transaction(
        &self,
        transaction_id: &str,
        data: SaveTransaction,
    ) -> Result<TransactionDetail> {
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

    async fn get_scheduled_transactions_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<ScheduledTransactionsDetailDelta> {
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

    pub async fn get_scheduled_transactions(&self) -> Result<Vec<ScheduledTransactionDetail>> {
        Ok(self
            .get_scheduled_transactions_request(None)
            .await?
            .scheduled_transactions)
    }

    pub async fn get_scheduled_transactions_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> Result<ScheduledTransactionsDetailDelta> {
        self.get_scheduled_transactions_request(last_knowledge_of_server)
            .await
    }

    pub async fn get_scheduled_transaction_by_id(
        &self,
        scheduled_transaction_id: &str,
    ) -> Result<ScheduledTransactionDetail> {
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

    /// Builds the ReqwestClient with some default headers staying the same for all requests.
    fn build_http_client() -> Result<ReqwestClient> {
        let mut headers = header::HeaderMap::new();
        let cargo_pkg_name = env!("CARGO_PKG_NAME");
        let cargo_pkg_version = env!("CARGO_PKG_VERSION");
        let user_agent_value = format!("{}/{}", cargo_pkg_name, cargo_pkg_version);
        headers.insert(
            reqwest::header::USER_AGENT,
            header::HeaderValue::from_str(&user_agent_value).unwrap(),
        );

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(http_client)
    }

    /// Builds a `GET` request by joining the path to `ynab_base_url` and setting `ynab_api_token` as a bearer_auth token.
    fn get(&self, path: &str) -> RequestBuilder {
        self.http_client
            .get(self.ynab_base_url.join(path).unwrap())
            .bearer_auth(self.ynab_api_token.as_str())
    }

    /// Builds a `GET` request by joining the path to `ynab_base_url` and setting `ynab_api_token` as a bearer_auth token.
    /// Also adds some query string to the request.
    /// # Example
    ///
    /// ```rust
    /// # use ynab::Result;
    /// # use ynab::Client;
    /// # async fn run() -> Result<()> {
    /// let body = get_with_query("path/to/resource", &[("key", "val")]).send().await?.text().await?;
    /// # Ok(())
    /// # }
    /// ```
    /// Calling `get_with_query("path/to/resource", &[("foo", "a"), ("boo", "b")])` gives `"path/to/resource?foo=a&boo=b"`
    fn get_with_query<T: Serialize + ?Sized>(&self, path: &str, query: &T) -> RequestBuilder {
        self.http_client
            .get(self.ynab_base_url.join(path).unwrap())
            .query(query)
            .bearer_auth(self.ynab_api_token.as_str())
    }

    /// Builds a `POST` request by joining the path to `ynab_base_url` and setting the body (if present) as json data.
    /// Also sets `ynab_api_token` as a bearer_auth token.
    fn post<T>(&self, path: &str, body: Option<&T>) -> RequestBuilder
    where
        T: Serialize,
    {
        match body {
            Some(b) => self
                .http_client
                .post(self.ynab_base_url.join(path).unwrap())
                .bearer_auth(self.ynab_api_token.as_str())
                .json(b),
            None => self
                .http_client
                .post(self.ynab_base_url.join(path).unwrap())
                .bearer_auth(self.ynab_api_token.as_str()),
        }
    }

    /// Builds a `PATCH` request by joining the path to `ynab_base_url` and setting the body as json data.
    /// Also sets `ynab_api_token` as a bearer_auth token.
    fn patch<T>(&self, path: &str, body: &T) -> RequestBuilder
    where
        T: Serialize,
    {
        self.http_client
            .patch(self.ynab_base_url.join(path).unwrap())
            .bearer_auth(self.ynab_api_token.as_str())
            .json(body)
    }

    /// Builds a `PUT` request by joining the path to `ynab_base_url` and setting the body as json data.
    /// Also sets `ynab_api_token` as a bearer_auth token.
    fn put<T>(&self, path: &str, body: &T) -> RequestBuilder
    where
        T: Serialize,
    {
        self.http_client
            .put(self.ynab_base_url.join(path).unwrap())
            .bearer_auth(self.ynab_api_token.as_str())
            .json(body)
    }

    /// Returns the `default_budget_id` if previously set, or the special
    /// string `"last-used"` to be used in YNAB's API.
    fn get_budget_id(&self) -> &str {
        match self.default_budget_id.as_ref() {
            Some(b_id) => b_id,
            None => "last-used",
        }
    }

    /// Converts a string body into a rust's T representation of it.
    /// If the body contains an error from the api, it will return an `Error::Api()` enum.
    /// If the conversion fails using serde, it will return an `Error::Conversion()` enum.
    /// See https://github.com/serde-rs/json/issues/450#issuecomment-506505388 for an explanation on why T
    /// has to implement DeserializeOwned and not Deserialize<'de>
    fn convert_resp<T, B>(body: B) -> Result<T>
    where
        T: DeserializeOwned,
        B: AsRef<str>,
    {
        let resp: T = serde_json::from_str(body.as_ref()).map_err(|e| {
            let err = serde_json::from_str::<ApiErrorResponse>(body.as_ref());

            match err {
                Ok(api_err) => Error::Api(api_err.error),
                Err(_err) => Error::Conversion(e),
            }
        })?;

        Ok(resp)
    }
}
