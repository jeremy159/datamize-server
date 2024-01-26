use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use ynab::Category;

use crate::{CategoryConversionError, ExpenseCategorization};

#[derive(Debug)]
struct Expected {
    res: Result<ExpenseCategorization, CategoryConversionError>,
}

#[track_caller]
fn check_method(cat_group: Category, Expected { res }: Expected) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    assert_eq!(res, cat_group.try_into());
}

#[test]
fn correctly_converts_cat_group() {
    let category = Category {
        hidden: false,
        deleted: false,
        ..Faker.fake()
    };

    let expected = Expected {
        res: Ok(ExpenseCategorization {
            id: category.category_group_id,
            name: category.category_group_name.clone(),
            ..Default::default()
        }),
    };

    check_method(category, expected);
}

#[test]
fn error_when_cat_group_is_deleted_or_hidden() {
    let cat_group = Category {
        hidden: false,
        deleted: true,
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = Category {
        hidden: true,
        deleted: false,
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = Category {
        hidden: true,
        deleted: true,
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryConversionError),
    };

    check_method(cat_group, expected);
}

#[test]
fn error_when_cat_group_name_is_one_of_the_unauthorized() {
    let cat_group = Category {
        category_group_name: String::from("Hidden Categories") + &Faker.fake::<String>(),
        hidden: false,
        deleted: false,
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = Category {
        category_group_name: String::from("Internal Master Category") + &Faker.fake::<String>(),
        hidden: false,
        deleted: false,
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryConversionError),
    };

    check_method(cat_group, expected);

    let cat_group = Category {
        category_group_name: String::from("Credit Card Payments"),
        hidden: false,
        deleted: false,
        ..Faker.fake()
    };

    let expected = Expected {
        res: Err(CategoryConversionError),
    };

    check_method(cat_group, expected);
}
