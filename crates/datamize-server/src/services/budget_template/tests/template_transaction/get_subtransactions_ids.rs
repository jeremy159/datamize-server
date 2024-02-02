use datamize_domain::{DatamizeScheduledTransaction, Uuid};
use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use ynab::ScheduledSubTransaction;

use crate::services::budget_template::TemplateTransactionService;

fn check_get(scheduled_transactions: &[DatamizeScheduledTransaction], expected_resp: Vec<Uuid>) {
    let actual =
        TemplateTransactionService::get_subtransactions_category_ids(scheduled_transactions);

    assert_eq!(actual, expected_resp);
}

#[test]
fn empty_when_no_transactions() {
    check_get(&[], vec![]);
}

#[test]
fn empty_when_no_subtransactions() {
    check_get(
        &[
            DatamizeScheduledTransaction {
                subtransactions: vec![],
                ..Faker.fake()
            },
            DatamizeScheduledTransaction {
                subtransactions: vec![],
                ..Faker.fake()
            },
        ],
        vec![],
    );
}

#[test]
fn empty_when_no_subtransactions_with_category_id() {
    check_get(
        &[
            DatamizeScheduledTransaction {
                subtransactions: vec![
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: false,
                        ..Faker.fake()
                    },
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: false,
                        ..Faker.fake()
                    },
                ],
                ..Faker.fake()
            },
            DatamizeScheduledTransaction {
                subtransactions: vec![
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: false,
                        ..Faker.fake()
                    },
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: false,
                        ..Faker.fake()
                    },
                ],
                ..Faker.fake()
            },
        ],
        vec![],
    );
}

#[test]
fn filters_deleted_subtransactions_with_category_id() {
    check_get(
        &[
            DatamizeScheduledTransaction {
                subtransactions: vec![
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: true,
                        ..Faker.fake()
                    },
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: false,
                        ..Faker.fake()
                    },
                ],
                ..Faker.fake()
            },
            DatamizeScheduledTransaction {
                subtransactions: vec![
                    ScheduledSubTransaction {
                        category_id: Some(Faker.fake()),
                        deleted: true,
                        ..Faker.fake()
                    },
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: false,
                        ..Faker.fake()
                    },
                ],
                ..Faker.fake()
            },
        ],
        vec![],
    );
}

#[test]
fn keeps_subtransaction_ids_that_pass_all_checks() {
    let cat_id1 = Faker.fake();
    let cat_id2 = Faker.fake();

    check_get(
        &[
            DatamizeScheduledTransaction {
                subtransactions: vec![
                    ScheduledSubTransaction {
                        category_id: None,
                        deleted: true,
                        ..Faker.fake()
                    },
                    ScheduledSubTransaction {
                        category_id: Some(cat_id1),
                        deleted: false,
                        ..Faker.fake()
                    },
                ],
                ..Faker.fake()
            },
            DatamizeScheduledTransaction {
                subtransactions: vec![
                    ScheduledSubTransaction {
                        category_id: Some(Faker.fake()),
                        deleted: true,
                        ..Faker.fake()
                    },
                    ScheduledSubTransaction {
                        category_id: Some(cat_id2),
                        deleted: false,
                        ..Faker.fake()
                    },
                ],
                ..Faker.fake()
            },
        ],
        vec![cat_id1, cat_id2],
    );
}
