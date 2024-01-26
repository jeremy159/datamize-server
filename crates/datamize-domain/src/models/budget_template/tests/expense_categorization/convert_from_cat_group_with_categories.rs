use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use ynab::CategoryGroupWithCategories;

use crate::{CategoryGroupWithCategoriesConversionError, ExpenseCategorization};

#[derive(Debug)]
struct Expected {
    res: Result<ExpenseCategorization, CategoryGroupWithCategoriesConversionError>,
}

#[track_caller]
fn check_method(cat_group: CategoryGroupWithCategories, Expected { res }: Expected) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    assert_eq!(res, cat_group.try_into());
}

#[test]
fn correctly_converts_cat_group() {
    let cat_group = CategoryGroupWithCategories {
        hidden: false,
        deleted: false,
        categories: vec![],
        ..Faker.fake()
    };

    let expected = Expected {
        res: Ok(ExpenseCategorization {
            id: cat_group.id,
            name: cat_group.name.clone(),
            ..Default::default()
        }),
    };

    check_method(cat_group, expected);
}

#[test]
fn error_when_cat_group_is_deleted_or_hidden() {
    let cat_group = CategoryGroupWithCategories {
        hidden: false,
        deleted: true,
        categories: vec![],
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryGroupWithCategoriesConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = CategoryGroupWithCategories {
        hidden: true,
        deleted: false,
        categories: vec![],
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryGroupWithCategoriesConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = CategoryGroupWithCategories {
        hidden: true,
        deleted: true,
        categories: vec![],
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryGroupWithCategoriesConversionError),
    };

    check_method(cat_group, expected);
}

#[test]
fn error_when_cat_group_name_is_one_of_the_unauthorized() {
    let cat_group = CategoryGroupWithCategories {
        name: String::from("Hidden Categories") + &Faker.fake::<String>(),
        hidden: false,
        deleted: false,
        categories: vec![],
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryGroupWithCategoriesConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = CategoryGroupWithCategories {
        name: String::from("Internal Master Category") + &Faker.fake::<String>(),
        hidden: false,
        deleted: false,
        categories: vec![],
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryGroupWithCategoriesConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = CategoryGroupWithCategories {
        name: String::from("Credit Card Payments"),
        hidden: false,
        deleted: false,
        categories: vec![],
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryGroupWithCategoriesConversionError),
    };

    check_method(cat_group, expected);
}
