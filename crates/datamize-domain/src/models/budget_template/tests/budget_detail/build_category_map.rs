use chrono::{DateTime, Datelike, Days, Local};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;
use ynab::{RecurFrequency, ScheduledSubTransaction};

use crate::{BudgetDetails, DatamizeScheduledTransaction};

#[derive(Debug, Clone)]
struct Expected {
    empty: bool,
    contains: Vec<(Uuid, Vec<DatamizeScheduledTransaction>)>,
}

#[track_caller]
fn check_method(
    scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    date: &DateTime<Local>,
    Expected { empty, contains }: Expected,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    let map =
        BudgetDetails::build_category_to_scheduled_transaction_map(scheduled_transactions, date);
    assert_eq!(map.is_empty(), empty, "map is not empty");

    for (cat_id, values) in contains {
        let key_values = map.get_key_value(&cat_id);
        assert!(key_values.is_some(), "key value pair is not defined");
        let (key, values_saved) = key_values.unwrap();
        assert_eq!(&cat_id, key, "category id is not equal to key");
        assert_eq!(
            &values, values_saved,
            "saved transactions are not what was expected"
        );
    }
}

#[test]
fn empty_when_no_transactions() {
    check_method(
        vec![],
        &Local::now(),
        Expected {
            empty: true,
            contains: vec![],
        },
    );
}

#[test]
fn empty_when_transaction_deleted() {
    let trans = DatamizeScheduledTransaction {
        date_first: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        category_id: Some(Faker.fake()),
        deleted: true,
        ..Faker.fake()
    };

    check_method(
        vec![trans],
        &Local::now(),
        Expected {
            empty: true,
            contains: vec![],
        },
    );
}

#[test]
fn empty_when_transaction_does_not_have_cat_id() {
    let trans = DatamizeScheduledTransaction {
        date_first: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: false,
        category_id: None,
        ..Faker.fake()
    };

    check_method(
        vec![trans],
        &Local::now(),
        Expected {
            empty: true,
            contains: vec![],
        },
    );
}

#[test]
fn sub_trans_replace_trans() {
    let category_id1 = Faker.fake();
    let sub_trans1 = ScheduledSubTransaction {
        deleted: false,
        category_id: Some(category_id1),
        ..Faker.fake()
    };
    let category_id2 = Faker.fake();
    let sub_trans2 = ScheduledSubTransaction {
        deleted: false,
        category_id: Some(category_id2),
        ..Faker.fake()
    };
    let trans = DatamizeScheduledTransaction {
        date_first: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![sub_trans1.clone(), sub_trans2.clone()],
        deleted: false,
        category_id: Some(Faker.fake()),
        ..Faker.fake()
    };

    let sub_trans1_to_trans = DatamizeScheduledTransaction {
        id: sub_trans1.id,
        amount: sub_trans1.amount,
        memo: sub_trans1.memo,
        payee_id: sub_trans1.payee_id,
        category_id: sub_trans1.category_id,
        deleted: sub_trans1.deleted,
        subtransactions: vec![],
        category_name: None,
        date_next: trans.date_first,
        ..trans.clone()
    };

    let sub_trans2_to_trans = DatamizeScheduledTransaction {
        id: sub_trans2.id,
        amount: sub_trans2.amount,
        memo: sub_trans2.memo,
        payee_id: sub_trans2.payee_id,
        category_id: sub_trans2.category_id,
        deleted: sub_trans2.deleted,
        subtransactions: vec![],
        category_name: None,
        date_next: trans.date_first,
        ..trans.clone()
    };

    check_method(
        vec![trans.clone()],
        &Local::now(),
        Expected {
            empty: false,
            contains: vec![
                (category_id1, vec![sub_trans1_to_trans]),
                (category_id2, vec![sub_trans2_to_trans]),
            ],
        },
    );
}

#[test]
fn all_trans_that_repeats_in_current_month() {
    let date_first = Local::now().date_naive().with_day(5).unwrap();
    let trans = DatamizeScheduledTransaction {
        date_first,
        date_next: date_first,
        frequency: RecurFrequency::Weekly,
        subtransactions: vec![],
        deleted: false,
        category_id: Some(Faker.fake()),
        ..Faker.fake()
    };

    let trans2 = DatamizeScheduledTransaction {
        date_next: trans.date_first.checked_add_days(Days::new(7)).unwrap(),
        ..trans.clone()
    };

    let trans3 = DatamizeScheduledTransaction {
        date_next: trans.date_first.checked_add_days(Days::new(14)).unwrap(),
        ..trans.clone()
    };

    let trans4 = DatamizeScheduledTransaction {
        date_next: trans.date_first.checked_add_days(Days::new(21)).unwrap(),
        ..trans.clone()
    };

    check_method(
        vec![trans.clone()],
        &Local::now(),
        Expected {
            empty: false,
            contains: vec![(
                trans.category_id.unwrap(),
                vec![trans, trans2, trans3, trans4],
            )],
        },
    );
}
