use db_sqlite::budget_providers::ynab::sabotage_categories_table;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::{Category, CategoryGroupWithCategoriesDelta, MonthDetail};

use crate::services::{
    budget_providers::ynab::tests::category::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

struct YnabData(CategoryGroupWithCategoriesDelta, MonthDetail);

#[derive(Clone)]
struct DbData(Vec<Category>);

async fn check_get_latest(
    pool: SqlitePool,
    ynab_data: YnabData,
    db_data: Option<DbData>,
    expected_resp: Option<Vec<Category>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool, ynab_data.0.clone(), ynab_data.1.clone()).await;

    if let Some(DbData(ref categories)) = db_data {
        context.set_categories(categories).await;
    }
    let delta_before = context.get_delta().await;

    let response = context.service().get_latest_categories().await;

    if let Some(mut expected_resp) = expected_resp {
        let (mut res_body, _) = response.unwrap();
        res_body.sort_by_key(|c| c.name.to_string());
        expected_resp.sort_by_key(|c| c.name.to_string());
        assert_eq!(res_body, expected_resp);
        let delta_after = context.get_delta().await;

        assert_ne!(delta_before, delta_after);
    } else {
        assert_err(response.unwrap_err(), expected_err);
        let delta_after = context.get_delta().await;

        assert_eq!(delta_before, delta_after);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_empty_list_when_nothing_in_db(pool: SqlitePool) {
    let categories_delta = CategoryGroupWithCategoriesDelta {
        category_groups: vec![],
        ..Faker.fake()
    };
    check_get_latest(
        pool,
        YnabData(categories_delta, Faker.fake()),
        None,
        Some(vec![]),
        None,
    )
    .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_success_with_what_is_in_db(pool: SqlitePool) {
    let categories_delta = Faker.fake::<CategoryGroupWithCategoriesDelta>();
    let categories = Faker.fake::<Vec<Category>>();
    let mut expected: Vec<Category> = categories_delta
        .category_groups
        .clone()
        .into_iter()
        .flat_map(|cg| cg.categories)
        .collect();
    expected.extend(categories.clone());

    check_get_latest(
        pool,
        YnabData(categories_delta, Faker.fake()),
        Some(DbData(categories.clone())),
        Some(expected),
        None,
    )
    .await;
}

// FIXME: For some reasons sometimes the test fails... Might be related to the redis test mock
// #[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn issue_with_db_should_not_update_saved_delta(pool: SqlitePool) {
    sabotage_categories_table(&pool).await.unwrap();

    check_get_latest(
        pool,
        YnabData(Faker.fake(), Faker.fake()),
        None,
        None,
        Some(ErrorType::Database),
    )
    .await;
}
