use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    client::Response, error::YnabResult, Category, CategoryGroupWithCategories,
    CategoryGroupWithCategoriesDelta, Client, SaveMonthCategory,
};

#[cfg_attr(any(feature = "testutils", test), mockall::automock)]
#[async_trait]
pub trait CategoryRequests {
    async fn get_categories(&self) -> YnabResult<Vec<CategoryGroupWithCategories>>;

    async fn get_categories_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<CategoryGroupWithCategoriesDelta>;

    async fn get_category_by_id(&self, category_id: &str) -> YnabResult<Category>;

    async fn get_category_by_id_for(&self, category_id: &str, month: &str) -> YnabResult<Category>;

    async fn update_category_for(
        &self,
        category_id: &str,
        month: &str,
        data: SaveMonthCategory,
    ) -> YnabResult<Category>;
}

#[async_trait]
impl CategoryRequests for Client {
    async fn get_categories(&self) -> YnabResult<Vec<CategoryGroupWithCategories>> {
        Ok(self.get_categories_request(None).await?.category_groups)
    }

    async fn get_categories_delta(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<CategoryGroupWithCategoriesDelta> {
        self.get_categories_request(last_knowledge_of_server).await
    }

    async fn get_category_by_id(&self, category_id: &str) -> YnabResult<Category> {
        self.get_category_by_id_request(category_id, None).await
    }

    async fn get_category_by_id_for(&self, category_id: &str, month: &str) -> YnabResult<Category> {
        self.get_category_by_id_request(category_id, Some(month))
            .await
    }

    async fn update_category_for(
        &self,
        category_id: &str,
        month: &str,
        data: SaveMonthCategory,
    ) -> YnabResult<Category> {
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
}

impl Client {
    async fn get_categories_request(
        &self,
        last_knowledge_of_server: Option<i64>,
    ) -> YnabResult<CategoryGroupWithCategoriesDelta> {
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

    async fn get_category_by_id_request(
        &self,
        category_id: &str,
        month: Option<&str>,
    ) -> YnabResult<Category> {
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
}
