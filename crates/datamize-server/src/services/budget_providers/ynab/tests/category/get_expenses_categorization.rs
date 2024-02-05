use datamize_domain::ExpenseCategorization;
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use sqlx::SqlitePool;
use ynab::CategoryGroup;

use crate::services::{
    budget_providers::ynab::tests::category::testutils::TestContext,
    testutils::{assert_err, ErrorType},
};

#[derive(Clone)]
struct DbData(Vec<ExpenseCategorization>);

async fn check_get_expenses_categorization<T: TryInto<ExpenseCategorization>>(
    pool: SqlitePool,
    categories: Vec<T>,
    db_data: Option<DbData>,
    expected_resp: Option<Vec<ExpenseCategorization>>,
    expected_err: Option<ErrorType>,
) {
    let context = TestContext::setup(pool, Faker.fake(), Faker.fake()).await;

    if let Some(DbData(ref expenses_categorization)) = db_data {
        context
            .set_expenses_categorization(expenses_categorization)
            .await;
    }

    let response = context
        .service()
        .get_expenses_categorization(categories)
        .await;

    if let Some(mut expected_resp) = expected_resp {
        let mut res_body = response.unwrap();
        res_body.sort_by_key(|ec| ec.name.to_string());
        expected_resp.sort_by_key(|ec| ec.name.to_string());
        assert_eq!(res_body, expected_resp);
    } else {
        assert_err(response.unwrap_err(), expected_err);
    }
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_empty_list_when_nothing_in_db(pool: SqlitePool) {
    check_get_expenses_categorization::<CategoryGroup>(pool, vec![], None, Some(vec![]), None)
        .await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_all_categorizations_from_categories(pool: SqlitePool) {
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

    let expected = category_groups
        .clone()
        .into_iter()
        .filter_map(|c| c.try_into().ok())
        .collect();

    check_get_expenses_categorization(pool, category_groups, None, Some(expected), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn returns_unique_categorizations_from_categories(pool: SqlitePool) {
    let cat_group = CategoryGroup {
        deleted: false,
        hidden: false,
        ..Faker.fake()
    };
    let category_groups = vec![cat_group.clone(), cat_group.clone()];

    let expected = vec![cat_group.try_into().unwrap()];

    check_get_expenses_categorization(pool, category_groups, None, Some(expected), None).await;
}

#[sqlx::test(migrations = "../db-sqlite/migrations")]
async fn should_use_existing_categorizations_if_found(pool: SqlitePool) {
    let expense_categorization = Faker.fake::<ExpenseCategorization>();

    let cat_group = CategoryGroup {
        deleted: false,
        hidden: false,
        id: expense_categorization.id,
        name: expense_categorization.name.clone(),
    };
    let category_groups = vec![cat_group.clone()];

    check_get_expenses_categorization(
        pool,
        category_groups,
        Some(DbData(vec![expense_categorization.clone()])),
        Some(vec![expense_categorization]),
        None,
    )
    .await;
}

// TODO: To handle this in code. Not yet handled
// #[sqlx::test(migrations = "../db-sqlite/migrations")]
// async fn should_use_existing_categorizations_plus_incoming(pool: SqlitePool) {
//     let expense_categorization = Faker.fake::<ExpenseCategorization>();

//     let expense_categorization_cloned = expense_categorization.clone();
//     let cat_group = CategoryGroup {
//         deleted: false,
//         hidden: false,
//         ..Faker.fake()
//     };
//     let category_groups = vec![cat_group.clone(), cat_group.clone()];

//     let mut expected = vec![cat_group.try_into().unwrap()];
//     expected.push(expense_categorization_cloned);

//     check_get_expenses_categorization(
//         pool,
//         category_groups,
//         Some(DbData(vec![expense_categorization])),
//         Some(expected),
//         None,
//     )
//     .await;
// }

// TODO: To add test case when db error is encountered
// #[sqlx::test(migrations = "../db-sqlite/migrations")]
// async fn issue_with_db_should_not_update_saved_delta(pool: SqlitePool) {
//     sabotage_scheduled_transactions_table(&pool).await.unwrap();

//     check_get_expenses_categorization(
//         pool,
//         YnabData(Faker.fake()),
//         None,
//         None,
//         Some(ErrorType::Internal),
//     )
//     .await;
// }
