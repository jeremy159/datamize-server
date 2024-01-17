use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use uuid::Uuid;
use ynab::SubTransaction;

use crate::DatamizeScheduledTransaction;

#[track_caller]
fn check_method(st: &DatamizeScheduledTransaction, expected_len: usize, expected_id: Uuid) {
    let caller_location = std::panic::Location::caller();
    let caller_line_number = caller_location.line();
    println!("check_method called from line: {}", caller_line_number);

    let trans = st.clone().flatten();
    assert_eq!(trans.len(), expected_len);
    if !trans.is_empty() {
        assert_eq!(trans[0].id, expected_id);
    }
}

#[test]
fn returns_only_itself_when_no_sub_transactions() {
    let st = DatamizeScheduledTransaction {
        subtransactions: vec![],
        ..Faker.fake()
    };

    check_method(&st, 1, st.id);
}

#[test]
fn returns_itself_when_all_sub_transactions_are_deleted() {
    let st = DatamizeScheduledTransaction {
        subtransactions: vec![SubTransaction {
            deleted: true,
            ..Faker.fake()
        }],
        ..Faker.fake()
    };

    check_method(&st, 1, st.id);
}

#[test]
fn returns_only_sub_trans_when_non_deleted() {
    let st = DatamizeScheduledTransaction {
        subtransactions: vec![SubTransaction {
            deleted: false,
            ..Faker.fake()
        }],
        ..Faker.fake()
    };

    check_method(&st, 1, st.subtransactions[0].id);
}
