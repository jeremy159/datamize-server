use fake::{Fake, Faker};
use pretty_assertions::assert_eq;
use ynab::Category;

use crate::{
    Budgeter, BudgeterExt, ComputedSalary, Expense, ExpenseCategorization, ExpenseType,
    ExternalExpense, PartiallyComputed, SubExpenseType, Uncomputed,
};

#[test]
fn correctly_converts_category_to_uncomputed_expense() {
    let category: Category = Faker.fake();
    let expense: Expense<Uncomputed> = category.clone().into();

    assert_eq!(category.id, expense.id());
    assert_eq!(&category.name, expense.name());
    assert!(!expense.is_external());
    assert_eq!(Some(&category), expense.category());

    // Default value for the other fields
    assert_eq!(expense.expense_type(), &ExpenseType::default());
    assert_eq!(expense.sub_expense_type(), &SubExpenseType::default());
    assert_eq!(expense.individual_associated(), Option::default());
    assert_eq!(expense.scheduled_transactions(), Vec::default());
}

#[test]
fn no_categorization_when_expense_does_not_have_category() {
    let external_expense = ExternalExpense {
        expense_type: Default::default(),
        sub_expense_type: Default::default(),
        ..Faker.fake()
    };
    let expense: Expense<PartiallyComputed> = external_expense.clone().into();
    let expenses_categorization = fake::vec![ExpenseCategorization; 1..5];
    let expense = expense.set_categorization(&expenses_categorization);

    assert_eq!(expense.expense_type(), &ExpenseType::default());
    assert_eq!(expense.sub_expense_type(), &SubExpenseType::default());
}

#[test]
fn no_categorization_when_category_does_not_match_any_group() {
    let category: Category = Faker.fake();
    let expense: Expense<Uncomputed> = category.clone().into();
    let expenses_categorization = fake::vec![ExpenseCategorization; 1..5];
    let expense = expense.set_categorization(&expenses_categorization);

    assert_eq!(expense.expense_type(), &ExpenseType::default());
    assert_eq!(expense.sub_expense_type(), &SubExpenseType::default());
}

#[test]
fn sets_type_and_sub_type_of_categorization_when_match() {
    let expenses_categorization = fake::vec![ExpenseCategorization; 1..5];
    let category = Category {
        category_group_id: expenses_categorization[0].id,
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.set_categorization(&expenses_categorization);

    assert_eq!(
        expense.expense_type(),
        &expenses_categorization[0].expense_type
    );
    assert_eq!(
        expense.sub_expense_type(),
        &expenses_categorization[0].sub_expense_type
    );
}

#[test]
fn no_individual_association_when_expense_does_not_have_budgeter_name() {
    let category: Category = Faker.fake();
    let expense: Expense<Uncomputed> = category.clone().into();
    let budgeters = fake::vec![Budgeter<ComputedSalary>; 1..5];
    let expense = expense.set_individual_association(&budgeters);

    assert_eq!(expense.individual_associated(), None);
}

#[test]
fn sets_individual_association_when_expense_does_have_budgeter_name() {
    let budgeters = fake::vec![Budgeter<ComputedSalary>; 1..5];
    let mut name = Faker.fake::<String>();
    name.push_str(budgeters[0].name());
    let category = Category {
        name,
        ..Faker.fake()
    };
    let expense: Expense<Uncomputed> = category.clone().into();
    let expense = expense.set_individual_association(&budgeters);

    assert_eq!(
        expense.individual_associated(),
        Some(&budgeters[0].name().to_owned())
    );
}
