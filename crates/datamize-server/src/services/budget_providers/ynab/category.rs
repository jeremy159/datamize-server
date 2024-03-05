use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, Datelike, Local, NaiveDate};
use datamize_domain::{
    async_trait,
    db::{
        ynab::{DynYnabCategoryMetaRepo, DynYnabCategoryRepo},
        DbError, DynExpenseCategorizationRepo,
    },
    ExpenseCategorization, MonthTarget,
};
use ynab::{Category, CategoryGroup, CategoryRequests, MonthRequests};

use crate::error::DatamizeResult;

#[async_trait]
pub trait CategoryServiceExt: Send + Sync {
    async fn get_categories_of_month(
        &self,
        month: MonthTarget,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)>;
}

pub type DynCategoryService = Arc<dyn CategoryServiceExt>;

pub struct CategoryService<YC: CategoryRequests + MonthRequests> {
    pub ynab_category_repo: DynYnabCategoryRepo,
    pub ynab_category_meta_repo: DynYnabCategoryMetaRepo,
    pub expense_categorization_repo: DynExpenseCategorizationRepo,
    pub ynab_client: Arc<YC>,
}

impl<YC> Clone for CategoryService<YC>
where
    YC: CategoryRequests + MonthRequests,
{
    fn clone(&self) -> Self {
        Self {
            ynab_category_repo: self.ynab_category_repo.clone(),
            ynab_category_meta_repo: self.ynab_category_meta_repo.clone(),
            expense_categorization_repo: self.expense_categorization_repo.clone(),
            ynab_client: self.ynab_client.clone(),
        }
    }
}

impl<YC> CategoryService<YC>
where
    YC: CategoryRequests + MonthRequests + Sync + Send,
{
    pub fn new_arced(
        ynab_category_repo: DynYnabCategoryRepo,
        ynab_category_meta_repo: DynYnabCategoryMetaRepo,
        expense_categorization_repo: DynExpenseCategorizationRepo,
        ynab_client: Arc<YC>,
    ) -> Arc<Self> {
        Arc::new(CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client,
        })
    }

    pub(crate) async fn get_latest_categories(
        &self,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
        self.check_last_saved().await?;
        let saved_categories_delta = self.ynab_category_meta_repo.get_delta().await.ok();

        let category_groups_with_categories_delta = self
            .ynab_client
            .get_categories_delta(saved_categories_delta)
            .await?;

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

        self.ynab_category_repo.update_all(&categories).await?;

        self.ynab_category_meta_repo
            .set_delta(category_groups_with_categories_delta.server_knowledge)
            .await?;

        Ok((
            self.ynab_category_repo.get_all().await?,
            expenses_categorization,
        ))
    }

    pub(crate) async fn check_last_saved(&self) -> DatamizeResult<()> {
        let current_date = Local::now().date_naive();
        if let Ok(last_saved) = self.ynab_category_meta_repo.get_last_saved().await {
            let last_saved_date: NaiveDate = last_saved.parse()?;
            if current_date.month() != last_saved_date.month() {
                tracing::debug!(
                    ?current_date,
                    ?last_saved_date,
                    "discarding knowledge_server",
                );
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

        Ok(())
    }

    pub(crate) async fn get_expenses_categorization<T: TryInto<ExpenseCategorization>>(
        &self,
        categories: Vec<T>,
    ) -> DatamizeResult<Vec<ExpenseCategorization>> {
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
                    Err(DbError::NotFound) => {
                        self.expense_categorization_repo.update(&ec).await?;
                        ec
                    }
                    Err(e) => return Err(e.into()),
                };

                expenses_categorization_set.insert(expense_categorization);
            }
        }

        // TODO: Quick fix, for now also populate what we have in DB, but figure something better.
        // The issue is if we have a saved delta, we won't get the full list of categories from YNAB,
        // hence neither of the expenses categorization.
        for ec in self.expense_categorization_repo.get_all().await? {
            expenses_categorization_set.insert(ec);
        }

        Ok(expenses_categorization_set.into_iter().collect())
    }
}

#[async_trait]
impl<YC> CategoryServiceExt for CategoryService<YC>
where
    YC: CategoryRequests + MonthRequests + Sync + Send,
{
    #[tracing::instrument(skip(self))]
    async fn get_categories_of_month(
        &self,
        month: MonthTarget,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
        match month {
            MonthTarget::Previous | MonthTarget::Next => {
                let categories = self
                    .ynab_client
                    .get_month_by_date(&DateTime::<Local>::from(month).date_naive().to_string())
                    .await
                    .map(|month_detail| month_detail.categories)?;

                let expenses_categorization =
                    self.get_expenses_categorization(categories.clone()).await?;

                Ok((categories, expenses_categorization))
            }
            MonthTarget::Current => self.get_latest_categories().await,
        }
    }
}
