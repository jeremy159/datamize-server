use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use ynab::{Category, CategoryGroup, Client};

use crate::{
    db::{
        budget_providers::ynab::{YnabCategoryMetaRepo, YnabCategoryRepo},
        budget_template::ExpenseCategorizationRepo,
    },
    error::{AppError, DatamizeResult},
    models::budget_template::{ExpenseCategorization, MonthTarget},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CategoryServiceExt {
    async fn get_categories_of_month(
        &mut self,
        month: MonthTarget,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)>;
}

pub struct CategoryService<
    YCR: YnabCategoryRepo,
    YCMR: YnabCategoryMetaRepo,
    ECR: ExpenseCategorizationRepo,
> {
    pub ynab_category_repo: YCR,
    pub ynab_category_meta_repo: YCMR,
    pub expense_categorization_repo: ECR,
    pub ynab_client: Arc<Client>,
}

#[async_trait]
impl<YCR, YCMR, ECR> CategoryServiceExt for CategoryService<YCR, YCMR, ECR>
where
    YCR: YnabCategoryRepo + Sync + Send,
    YCMR: YnabCategoryMetaRepo + Sync + Send,
    ECR: ExpenseCategorizationRepo + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_categories_of_month(
        &mut self,
        month: MonthTarget,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
        match month {
            MonthTarget::Previous | MonthTarget::Next => {
                let categories = self
                    .ynab_client
                    .get_month_by_date(&DateTime::<Local>::from(month).date_naive().to_string())
                    .await
                    .map_err(anyhow::Error::from)
                    .map(|month_detail| month_detail.categories)?;

                let expenses_categorization =
                    self.get_expenses_categorization(categories.clone()).await?;

                Ok((categories, expenses_categorization))
            }
            MonthTarget::Current => self.get_latest_categories().await,
        }
    }
}

impl<YCR, YCMR, ECR> CategoryService<YCR, YCMR, ECR>
where
    YCR: YnabCategoryRepo + Sync + Send,
    YCMR: YnabCategoryMetaRepo + Sync + Send,
    ECR: ExpenseCategorizationRepo + Sync + Send,
{
    async fn get_latest_categories(
        &mut self,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
        let current_date = Local::now().date_naive();
        if let Ok(last_saved) = self.ynab_category_meta_repo.get_last_saved().await {
            let last_saved_date: NaiveDate = last_saved.parse()?;
            if current_date.month() != last_saved_date.month() {
                // Discard knowledge_server when changing month.
                self.ynab_category_meta_repo.del_delta().await?;
                self.ynab_category_meta_repo
                    .set_last_saved(current_date.to_string())
                    .await?;
            }
        } else {
            self.ynab_category_meta_repo
                .set_last_saved(current_date.to_string())
                .await?;
        }
        let saved_categories_delta = self.ynab_category_meta_repo.get_delta().await.ok();

        let category_groups_with_categories_delta = self
            .ynab_client
            .get_categories_delta(saved_categories_delta)
            .await
            .context("failed to get categories from ynab's API")?;

        let (category_groups, categories): (Vec<CategoryGroup>, Vec<Vec<Category>>) =
            category_groups_with_categories_delta
                .category_groups
                .into_iter()
                .map(|cg| {
                    (
                        CategoryGroup {
                            id: cg.id,
                            name: cg.name,
                            hidden: cg.hidden,
                            deleted: cg.deleted,
                        },
                        cg.categories,
                    )
                })
                .unzip();

        let categories = categories.into_iter().flatten().collect::<Vec<_>>();

        let expenses_categorization = self.get_expenses_categorization(category_groups).await?;

        self.ynab_category_repo
            .update_all(&categories)
            .await
            .context("failed to save categories in database")?;

        self.ynab_category_meta_repo
            .set_delta(category_groups_with_categories_delta.server_knowledge)
            .await
            .context("failed to save last known server knowledge of categories in redis")?;

        Ok((
            self.ynab_category_repo
                .get_all()
                .await
                .context("failed to get categories from database")?,
            expenses_categorization,
        ))
    }

    async fn get_expenses_categorization<T: TryInto<ExpenseCategorization>>(
        &self,
        categories: Vec<T>,
    ) -> anyhow::Result<Vec<ExpenseCategorization>> {
        let mut expenses_categorization_set = HashSet::<ExpenseCategorization>::new();

        let expenses_categorization = categories
            .into_iter()
            .flat_map(TryInto::try_into)
            .collect::<Vec<_>>();

        for ec in expenses_categorization {
            if !expenses_categorization_set.contains(&ec) {
                let expense_categorization = match self.expense_categorization_repo.get(ec.id).await
                {
                    // TODO: Make sure to delete those newly hidden or deleted (Applies to all data coming from YNAB)
                    Ok(ec) => ec,
                    Err(AppError::ResourceNotFound) => {
                        self.expense_categorization_repo.update(&ec).await?;
                        ec
                    }
                    Err(e) => return Err(e.into()),
                };

                expenses_categorization_set.insert(expense_categorization);
            }
        }

        Ok(expenses_categorization_set.into_iter().collect())
    }
}
