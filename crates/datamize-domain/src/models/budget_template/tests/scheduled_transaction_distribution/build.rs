use std::collections::HashMap;

use chrono::{Days, Local, NaiveDate};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;
use ynab::{RecurFrequency, ScheduledSubTransaction};

use crate::{CategoryIdToNameMap, DatamizeScheduledTransaction, ScheduledTransactionsDistribution};

#[derive(Debug, Clone)]
struct Expected {
    empty: bool,
    contains: Vec<(String, Vec<DatamizeScheduledTransaction>)>,
}

#[track_caller]
fn check_method(
    scheduled_transactions: Vec<DatamizeScheduledTransaction>,
    category_id_to_name_map: Option<CategoryIdToNameMap>,
    Expected { empty, contains }: Expected,
) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    let mut scheduled_distribution_builder =
        ScheduledTransactionsDistribution::builder(scheduled_transactions);

    if let Some(category_id_to_name_map) = category_id_to_name_map {
        scheduled_distribution_builder =
            scheduled_distribution_builder.with_category_map(category_id_to_name_map);
    }

    let scheduled_distribution = scheduled_distribution_builder.build();

    let map = scheduled_distribution.map();
    assert_eq!(map.is_empty(), empty);

    for (date, values) in contains {
        let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
        let key_values = map.get_key_value(&date);
        assert!(key_values.is_some());
        let (key, values_saved) = key_values.unwrap();
        assert_eq!(&date, key);
        assert_eq!(&values, values_saved);
    }
}

#[test]
fn empty_when_no_transactions() {
    check_method(
        vec![],
        Some(HashMap::new()),
        Expected {
            empty: true,
            contains: vec![],
        },
    );
}

#[test]
fn empty_when_transaction_deleted() {
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: true,
        ..Faker.fake()
    };

    check_method(
        vec![trans],
        Some(HashMap::new()),
        Expected {
            empty: true,
            contains: vec![],
        },
    );
}

#[test]
fn trans_present_once_when_not_repeating_in_month() {
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: false,
        ..Faker.fake()
    };

    check_method(
        vec![trans.clone()],
        Some(HashMap::new()),
        Expected {
            empty: false,
            contains: vec![(trans.date_next.to_string(), vec![trans])],
        },
    );
}

#[test]
fn adds_both_trans_due_on_same_date() {
    let date_next = Local::now()
        .date_naive()
        .checked_add_days(Days::new(1))
        .unwrap();
    let first_trans = DatamizeScheduledTransaction {
        date_next,
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: false,
        ..Faker.fake()
    };
    let sec_trans = DatamizeScheduledTransaction {
        date_next,
        frequency: RecurFrequency::Every4Months,
        subtransactions: vec![],
        deleted: false,
        ..Faker.fake()
    };

    check_method(
        vec![first_trans.clone(), sec_trans.clone()],
        Some(HashMap::new()),
        Expected {
            empty: false,
            contains: vec![(
                first_trans.date_next.to_string(),
                vec![first_trans, sec_trans],
            )],
        },
    );
}

#[test]
fn adds_all_occurence_of_trans_repeating_in_month() {
    let first_date = Local::now().date_naive();
    let trans = DatamizeScheduledTransaction {
        date_next: first_date,
        frequency: RecurFrequency::EveryOtherWeek,
        subtransactions: vec![],
        deleted: false,
        ..Faker.fake()
    };

    let second_date = first_date.checked_add_days(Days::new(14)).unwrap();
    let second_trans = DatamizeScheduledTransaction {
        date_next: second_date,
        ..trans.clone()
    };

    check_method(
        vec![trans.clone()],
        Some(HashMap::new()),
        Expected {
            empty: false,
            contains: vec![
                (first_date.to_string(), vec![trans]),
                (second_date.to_string(), vec![second_trans]),
            ],
        },
    );
}

#[test]
fn empty_when_date_is_too_far_in_future() {
    let date_next = Local::now()
        .date_naive()
        .checked_add_days(Days::new(33))
        .unwrap();
    let trans = DatamizeScheduledTransaction {
        date_next,
        frequency: RecurFrequency::EveryOtherMonth,
        subtransactions: vec![],
        deleted: false,
        ..Faker.fake()
    };

    check_method(
        vec![trans.clone()],
        Some(HashMap::new()),
        Expected {
            empty: true,
            contains: vec![],
        },
    );
}

#[test]
fn only_trans_when_sub_trans_are_deleted() {
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![
            ScheduledSubTransaction {
                deleted: true,
                ..Faker.fake()
            },
            ScheduledSubTransaction {
                deleted: true,
                ..Faker.fake()
            },
        ],
        deleted: false,
        ..Faker.fake()
    };

    check_method(
        vec![trans.clone()],
        Some(HashMap::new()),
        Expected {
            empty: false,
            contains: vec![(trans.date_next.to_string(), vec![trans])],
        },
    );
}

#[test]
fn sub_trans_replace_trans() {
    let sub_trans1 = ScheduledSubTransaction {
        deleted: false,
        ..Faker.fake()
    };
    let sub_trans2 = ScheduledSubTransaction {
        deleted: false,
        ..Faker.fake()
    };
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![sub_trans1.clone(), sub_trans2.clone()],
        deleted: false,
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
        ..trans.clone()
    };

    check_method(
        vec![trans.clone()],
        Some(HashMap::new()),
        Expected {
            empty: false,
            contains: vec![(
                trans.date_next.to_string(),
                vec![sub_trans1_to_trans, sub_trans2_to_trans],
            )],
        },
    );
}

#[test]
fn no_category_name_when_no_cat_id() {
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: false,
        category_name: None,
        category_id: None,
        ..Faker.fake()
    };

    check_method(
        vec![trans.clone()],
        Some(HashMap::new()),
        Expected {
            empty: false,
            contains: vec![(trans.date_next.to_string(), vec![trans])],
        },
    );
}

#[test]
fn use_category_name_found_in_cat_id_to_name_map() {
    let cat_id: Uuid = Faker.fake();
    let cat_name: String = Faker.fake();
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: false,
        category_name: None,
        category_id: Some(cat_id),
        ..Faker.fake()
    };

    let category_id_to_name_map = HashMap::from([(cat_id, cat_name.clone())]);

    let updated_trans = DatamizeScheduledTransaction {
        category_name: Some(cat_name),
        ..trans.clone()
    };

    check_method(
        vec![trans.clone()],
        Some(category_id_to_name_map),
        Expected {
            empty: false,
            contains: vec![(trans.date_next.to_string(), vec![updated_trans])],
        },
    );
}

#[test]
fn no_category_name_when_not_found_in_cat_id_to_name_map() {
    let cat_id: Uuid = Faker.fake();
    let cat_name: String = Faker.fake();
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: false,
        category_name: None,
        category_id: Some(cat_id),
        ..Faker.fake()
    };

    let category_id_to_name_map = HashMap::from([(Faker.fake(), cat_name.clone())]);

    check_method(
        vec![trans.clone()],
        Some(category_id_to_name_map),
        Expected {
            empty: false,
            contains: vec![(trans.date_next.to_string(), vec![trans])],
        },
    );
}

#[test]
fn no_category_name_when_cat_id_to_name_map_not_defined() {
    let cat_id: Uuid = Faker.fake();
    let trans = DatamizeScheduledTransaction {
        date_next: Local::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap(),
        frequency: RecurFrequency::Every3Months,
        subtransactions: vec![],
        deleted: false,
        category_name: None,
        category_id: Some(cat_id),
        ..Faker.fake()
    };

    check_method(
        vec![trans.clone()],
        None,
        Expected {
            empty: false,
            contains: vec![(trans.date_next.to_string(), vec![trans])],
        },
    );
}
