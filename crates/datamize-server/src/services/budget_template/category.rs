use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use datamize_domain::{
    async_trait,
    db::{
        ynab::{DynYnabCategoryMetaRepo, DynYnabCategoryRepo},
        DbError, DynExpenseCategorizationRepo,
    },
    ExpenseCategorization, MonthTarget,
};
use dyn_clone::{clone_trait_object, DynClone};
use ynab::{Category, CategoryGroup, CategoryRequests, MonthRequests};

use crate::error::DatamizeResult;

#[async_trait]
pub trait CategoryServiceExt: DynClone + Send + Sync {
    async fn get_categories_of_month(
        &mut self,
        month: MonthTarget,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)>;
}

clone_trait_object!(CategoryServiceExt);

pub type DynCategoryService = Box<dyn CategoryServiceExt>;

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
    pub fn new_boxed(
        ynab_category_repo: DynYnabCategoryRepo,
        ynab_category_meta_repo: DynYnabCategoryMetaRepo,
        expense_categorization_repo: DynExpenseCategorizationRepo,
        ynab_client: Arc<YC>,
    ) -> Box<Self> {
        Box::new(CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client,
        })
    }

    async fn get_latest_categories(
        &mut self,
    ) -> DatamizeResult<(Vec<Category>, Vec<ExpenseCategorization>)> {
        self.check_last_saved().await?;
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

    async fn check_last_saved(&mut self) -> DatamizeResult<()> {
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
                    Err(DbError::NotFound) => {
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

#[async_trait]
impl<YC> CategoryServiceExt for CategoryService<YC>
where
    YC: CategoryRequests + MonthRequests + Sync + Send,
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

#[cfg(test)]
mod tests {
    use chrono::Months;
    use datamize_domain::db::{
        ynab::{MockYnabCategoryMetaRepo, MockYnabCategoryRepoImpl},
        MockExpenseCategorizationRepoImpl,
    };
    use fake::{Fake, Faker};
    use mockall::{mock, predicate::eq};
    use ynab::{
        Category, CategoryGroupWithCategories, CategoryGroupWithCategoriesDelta, MonthDetail,
        MonthSummary, MonthSummaryDelta, SaveMonthCategory, YnabResult,
    };

    use super::*;

    mock! {
        YnabClient {}
        #[async_trait]
        impl CategoryRequests for YnabClient {
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
        impl MonthRequests for YnabClient {
            async fn get_months(&self) -> YnabResult<Vec<MonthSummary>>;

            async fn get_months_delta(
                &self,
                last_knowledge_of_server: Option<i64>,
            ) -> YnabResult<MonthSummaryDelta>;

            async fn get_month_by_date(&self, date: &str) -> YnabResult<MonthDetail>;
        }
    }

    #[tokio::test]
    async fn check_last_saved_when_nothing_currently_saved_should_update_last_saved() {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());

        // ynab_category_meta_repo
        //     .expect_get_last_saved()
        //     .once()
        //     .returning(|| Err(DbError::NotFound));

        // let expected = Local::now().date_naive();
        // ynab_category_meta_repo
        //     .expect_set_last_saved()
        //     .once()
        //     .with(eq(expected.to_string()))
        //     .returning(|_| Ok(()));

        let ynab_client = MockYnabClient::new();

        let mut category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = category_service.check_last_saved().await;
        assert!(matches!(actual, Ok(())));
    }

    #[tokio::test]
    async fn check_last_saved_when_saved_date_is_the_same_month_as_current_should_not_update_last_saved(
    ) {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());

        // let saved_date = Local::now().date_naive();
        // ynab_category_meta_repo
        //     .expect_get_last_saved()
        //     .once()
        //     .returning(move || Ok(saved_date.to_string()));

        // ynab_category_meta_repo.expect_set_last_saved().never();

        let ynab_client = MockYnabClient::new();

        let mut category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = category_service.check_last_saved().await;
        assert!(matches!(actual, Ok(())));
    }

    #[tokio::test]
    async fn check_last_saved_when_saved_date_is_not_the_same_month_as_current_should_update_last_saved_and_delete_delta(
    ) {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());

        // let saved_date = Local::now()
        //     .date_naive()
        //     .checked_sub_months(Months::new(1))
        //     .unwrap();
        // ynab_category_meta_repo
        //     .expect_get_last_saved()
        //     .once()
        //     .returning(move || Ok(saved_date.to_string()));

        // let expected = Local::now().date_naive();
        // ynab_category_meta_repo
        //     .expect_set_last_saved()
        //     .once()
        //     .with(eq(expected.to_string()))
        //     .returning(|_| Ok(()));

        // ynab_category_meta_repo
        //     .expect_del_delta()
        //     .once()
        //     .returning(|| Ok(Faker.fake()));

        let ynab_client = MockYnabClient::new();

        let mut category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let actual = category_service.check_last_saved().await;
        assert!(matches!(actual, Ok(())));
    }

    #[tokio::test]
    async fn get_expenses_categorization_returns_all_categorizations_from_categories() {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());

        expense_categorization_repo
            .expect_get()
            .times(2)
            .returning(|_| Err(DbError::NotFound));
        expense_categorization_repo
            .expect_update()
            .times(2)
            .returning(|_| Ok(()));

        let ynab_client = MockYnabClient::new();

        let category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let category_groups = vec![
            CategoryGroup {
                deleted: false,
                hidden: false,
                ..Faker.fake()
            },
            CategoryGroup {
                deleted: false,
                hidden: false,
                ..Faker.fake()
            },
        ];

        let actual = category_service
            .get_expenses_categorization(category_groups.clone())
            .await;
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.len(), 2);
        assert!(actual.contains(&category_groups[0].clone().try_into().unwrap()));
        assert!(actual.contains(&category_groups[1].clone().try_into().unwrap()));
    }

    #[tokio::test]
    async fn get_expenses_categorization_returns_unique_categorizations_from_categories() {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());

        expense_categorization_repo
            .expect_get()
            .times(1)
            .returning(|_| Err(DbError::NotFound));
        expense_categorization_repo
            .expect_update()
            .times(1)
            .returning(|_| Ok(()));

        let ynab_client = MockYnabClient::new();

        let category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let cat_group = CategoryGroup {
            deleted: false,
            hidden: false,
            ..Faker.fake()
        };
        let category_groups = vec![cat_group.clone(), cat_group.clone()];

        let actual = category_service
            .get_expenses_categorization(category_groups.clone())
            .await;
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.len(), 1);
        assert!(actual.contains(&cat_group.try_into().unwrap()));
    }

    #[tokio::test]
    async fn get_expenses_categorization_should_use_existing_categorizations_if_found() {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());

        let expense_categorization = Faker.fake::<ExpenseCategorization>();

        let expense_categorization_cloned = expense_categorization.clone();
        expense_categorization_repo
            .expect_get()
            .once()
            .returning(move |_| Ok(expense_categorization_cloned.clone()));
        expense_categorization_repo.expect_update().never();

        let ynab_client = MockYnabClient::new();

        let category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        let cat_group = CategoryGroup {
            id: expense_categorization.id,
            name: expense_categorization.name.clone(),
            deleted: false,
            hidden: false,
        };
        let category_groups = vec![cat_group.clone()];

        let actual = category_service
            .get_expenses_categorization(category_groups.clone())
            .await;
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.len(), 1);
        assert!(actual.contains(&expense_categorization));
    }

    #[tokio::test]
    async fn get_latest_categories_should_return_all_categories() {
        let mut ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());
        let mut ynab_client = MockYnabClient::new();

        // let saved_date = Local::now().date_naive();
        // ynab_category_meta_repo
        //     .expect_get_last_saved()
        //     .once()
        //     .returning(move || Ok(saved_date.to_string()));

        // ynab_category_meta_repo
        //     .expect_get_delta()
        //     .once()
        //     .returning(move || Err(DbError::NotFound));

        let expected = CategoryGroupWithCategoriesDelta {
            server_knowledge: Faker.fake(),
            category_groups: vec![
                CategoryGroupWithCategories {
                    deleted: false,
                    hidden: false,
                    ..Faker.fake()
                },
                CategoryGroupWithCategories {
                    deleted: false,
                    hidden: false,
                    ..Faker.fake()
                },
            ],
        };
        let expected_cloned = expected.clone();
        ynab_client
            .expect_get_categories_delta()
            .once()
            .returning(move |_| Ok(expected_cloned.clone()));

        expense_categorization_repo
            .expect_get()
            .returning(move |_| Ok(Faker.fake()));

        let expected_categories = expected
            .category_groups
            .clone()
            .into_iter()
            .flat_map(|cg| cg.categories)
            .collect::<Vec<_>>();
        ynab_category_repo
            .expect_update_all()
            .once()
            .with(eq(expected_categories.clone()))
            .returning(|_| Ok(()));
        // ynab_category_meta_repo
        //     .expect_set_delta()
        //     .once()
        //     .with(eq(expected.server_knowledge))
        //     .returning(|_| Ok(()));

        let expected_categories_cloned = expected_categories.clone();
        ynab_category_repo
            .expect_get_all()
            .once()
            .return_once(move || Ok(expected_categories_cloned));

        let mut category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        category_service.get_latest_categories().await.unwrap();
    }

    #[tokio::test]
    async fn get_categories_of_month_for_current_month_should_go_through_get_latest_categories() {
        let mut ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());
        let mut ynab_client = MockYnabClient::new();

        // let saved_date = Local::now().date_naive();
        // ynab_category_meta_repo
        //     .expect_get_last_saved()
        //     .once()
        //     .returning(move || Ok(saved_date.to_string()));

        // ynab_category_meta_repo
        //     .expect_get_delta()
        //     .once()
        //     .returning(move || Err(DbError::NotFound));

        let expected = CategoryGroupWithCategoriesDelta {
            server_knowledge: Faker.fake(),
            category_groups: vec![CategoryGroupWithCategories {
                deleted: false,
                hidden: false,
                ..Faker.fake()
            }],
        };
        let expected_cloned = expected.clone();
        ynab_client
            .expect_get_categories_delta()
            .once()
            .returning(move |_| Ok(expected_cloned.clone()));

        expense_categorization_repo
            .expect_get()
            .returning(move |_| Ok(Faker.fake()));

        let expected_categories = expected
            .category_groups
            .clone()
            .into_iter()
            .flat_map(|cg| cg.categories)
            .collect::<Vec<_>>();
        ynab_category_repo
            .expect_update_all()
            .once()
            .with(eq(expected_categories.clone()))
            .returning(|_| Ok(()));
        // ynab_category_meta_repo
        //     .expect_set_delta()
        //     .once()
        //     .with(eq(expected.server_knowledge))
        //     .returning(|_| Ok(()));

        let expected_categories_cloned = expected_categories.clone();
        ynab_category_repo
            .expect_get_all()
            .once()
            .return_once(move || Ok(expected_categories_cloned));

        let mut category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        category_service
            .get_categories_of_month(MonthTarget::Current)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_categories_of_month_for_previous_month_should_use_get_month_by_date() {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());
        let mut ynab_client = MockYnabClient::new();

        let expected_date = Local::now()
            .checked_sub_months(Months::new(1))
            .unwrap()
            .date_naive()
            .to_string();
        let expected = MonthDetail {
            month: expected_date.clone(),
            deleted: false,
            ..Faker.fake()
        };
        let expected_cloned = expected.clone();
        ynab_client
            .expect_get_month_by_date()
            .once()
            .returning(move |_| Ok(expected_cloned.clone()));

        expense_categorization_repo
            .expect_get()
            .returning(move |_| Ok(Faker.fake()));

        let mut category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        category_service
            .get_categories_of_month(MonthTarget::Previous)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_categories_of_month_for_next_month_should_use_get_month_by_date() {
        let ynab_category_repo = Box::new(MockYnabCategoryRepoImpl::new());
        let ynab_category_meta_repo = MockYnabCategoryMetaRepo::new_boxed();
        let mut expense_categorization_repo = Box::new(MockExpenseCategorizationRepoImpl::new());
        let mut ynab_client = MockYnabClient::new();

        let expected_date = Local::now()
            .checked_add_months(Months::new(1))
            .unwrap()
            .date_naive()
            .to_string();
        let expected = MonthDetail {
            month: expected_date.clone(),
            deleted: false,
            ..Faker.fake()
        };
        let expected_cloned = expected.clone();
        ynab_client
            .expect_get_month_by_date()
            .once()
            .returning(move |_| Ok(expected_cloned.clone()));

        expense_categorization_repo
            .expect_get()
            .returning(move |_| Ok(Faker.fake()));

        let mut category_service = CategoryService {
            ynab_category_repo,
            ynab_category_meta_repo,
            expense_categorization_repo,
            ynab_client: Arc::new(ynab_client),
        };

        category_service
            .get_categories_of_month(MonthTarget::Next)
            .await
            .unwrap();
    }
}
