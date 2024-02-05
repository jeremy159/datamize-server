use datamize_domain::MonthTarget;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::{Category, CategoryGroupWithCategoriesDelta, MonthDetail};

use crate::services::budget_providers::{
    ynab::tests::category::testutils::TestContext, CategoryServiceExt,
};

struct YnabData(CategoryGroupWithCategoriesDelta, MonthDetail);

#[derive(Clone)]
struct DbData(Vec<Category>);

async fn check_get_categories_of_month(
    pool: SqlitePool,
    month: MonthTarget,
    ynab_data: YnabData,
    db_data: Option<DbData>,
    mut expected_resp: Vec<Category>,
) {
    let context = TestContext::setup(pool, ynab_data.0.clone(), ynab_data.1.clone()).await;

    if let Some(DbData(ref categories)) = db_data {
        context.set_categories(categories).await;
    }

    let response = context.service().get_categories_of_month(month).await;

    let (mut res_body, _) = response.unwrap();
    res_body.sort_by_key(|c| c.name.to_string());
    expected_resp.sort_by_key(|c| c.name.to_string());
    assert_eq!(res_body, expected_resp);
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn for_current_month_should_use_categories_endpoint(pool: SqlitePool) {
    let categories_delta = Faker.fake::<CategoryGroupWithCategoriesDelta>();
    let categories = Faker.fake::<Vec<Category>>();
    let mut expected: Vec<Category> = categories_delta
        .category_groups
        .clone()
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect();
    expected.extend(categories.clone());

    check_get_categories_of_month(
        pool,
        MonthTarget::Current,
        YnabData(categories_delta, Faker.fake()),
        Some(DbData(categories.clone())),
        expected,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn for_previous_month_should_use_month_detail_endpoint(pool: SqlitePool) {
    let month_detail = Faker.fake::<MonthDetail>();
    let categories = Faker.fake::<Vec<Category>>();
    let expected = month_detail.categories.clone();

    check_get_categories_of_month(
        pool,
        MonthTarget::Previous,
        YnabData(Faker.fake(), month_detail),
        Some(DbData(categories.clone())),
        expected,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn for_next_month_should_use_month_detail_endpoint(pool: SqlitePool) {
    let month_detail = Faker.fake::<MonthDetail>();
    let categories = Faker.fake::<Vec<Category>>();
    let expected = month_detail.categories.clone();

    check_get_categories_of_month(
        pool,
        MonthTarget::Next,
        YnabData(Faker.fake(), month_detail),
        Some(DbData(categories.clone())),
        expected,
    )
    .await;
}
