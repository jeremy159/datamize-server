use fake::{Fake, Faker};
use pretty_assertions::assert_eq;

use crate::{Expense, PartiallyComputed};

#[test]
fn proportion_is_0_when_total_is_0() {
    let expense: Expense<PartiallyComputed> = Faker.fake();
    let expense = expense.compute_proportions(0);

    assert_eq!(expense.projected_proportion(), 0.0);
    assert_eq!(expense.current_proportion(), 0.0);
}

#[test]
fn correctly_computes_proportions() {
    let expense: Expense<PartiallyComputed> = Faker.fake();
    let total = (1..1000).fake();
    let expense = expense.compute_proportions(total);

    assert_eq!(
        expense.projected_proportion(),
        expense.projected_amount() as f64 / total as f64
    );
    assert_eq!(
        expense.current_proportion(),
        expense.current_amount() as f64 / total as f64
    );
}
