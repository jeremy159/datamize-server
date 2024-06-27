use crate::{BalancePerYearPerMonth, MonthNum, YearlyBalances};
use pretty_assertions::{assert_eq, assert_ne};
use std::collections::BTreeMap;

// Define a struct that will implement the trait for testing purposes
#[derive(Debug)]
struct TestStruct {
    balances: BalancePerYearPerMonth,
}

// Implement the trait for the testing struct
impl YearlyBalances for TestStruct {
    fn balances(&self) -> &BalancePerYearPerMonth {
        &self.balances
    }

    fn balances_mut(&mut self) -> &mut BalancePerYearPerMonth {
        &mut self.balances
    }
}

#[test]
fn test_insert_balance() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, MonthNum::try_from(1).unwrap(), 100);
    assert_eq!(test_struct.get_balance(2022, MonthNum::January), Some(100));
}

#[test]
fn test_iter_balances() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 2_i16.try_into().unwrap(), 200);
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    let mut iterator = test_struct.iter_balances();
    assert_eq!(iterator.next(), Some((2022, MonthNum::January, 100)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::February, 200)));
    assert_eq!(iterator.next(), None);
}

#[test]
fn test_insert_balance_opt() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance_opt(2022, 1_i16.try_into().unwrap(), Some(100));
    assert_eq!(test_struct.get_balance(2022, MonthNum::January), Some(100));
}

#[test]
fn test_insert_balance_for_year() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    let balance_map: BTreeMap<MonthNum, Option<i64>> = [
        (MonthNum::January, Some(100)),
        (MonthNum::February, Some(200)),
    ]
    .iter()
    .cloned()
    .collect();
    test_struct.insert_balance_for_year(2022, balance_map.clone());
    assert_eq!(test_struct.get_balance(2022, MonthNum::January), Some(100));
    assert_eq!(test_struct.get_balance(2022, MonthNum::February), Some(200));
}

#[test]
fn test_iter_all_balances() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    test_struct.insert_balance_opt(2022, 2_i16.try_into().unwrap(), None);
    test_struct.insert_balance_opt(2022, 3_i16.try_into().unwrap(), None);
    let mut iterator = test_struct.iter_all_balances();
    assert_eq!(iterator.next(), Some((2022, MonthNum::January, Some(100))));
    assert_eq!(iterator.next(), Some((2022, MonthNum::February, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::March, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::April, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::May, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::June, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::July, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::August, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::September, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::October, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::November, None)));
    assert_eq!(iterator.next(), Some((2022, MonthNum::December, None)));
    assert_eq!(iterator.next(), None);
}

#[test]
fn test_iter_years() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2023, 1_i16.try_into().unwrap(), 200);
    let mut iterator = test_struct.iter_years();
    assert_eq!(iterator.next(), Some(2022));
    assert_eq!(iterator.next(), Some(2023));
    assert_eq!(iterator.next(), None);
}

#[test]
fn test_iter_months() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2023, 1_i16.try_into().unwrap(), 200);
    let mut iterator = test_struct.iter_months();
    assert_eq!(iterator.next(), Some((2022, MonthNum::January)));
    assert_eq!(iterator.next(), Some((2023, MonthNum::January)));
    assert_eq!(iterator.next(), None);
}

#[test]
fn test_get_first_month() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 3_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 200);
    assert_eq!(
        test_struct.get_first_month(),
        Some((2022, MonthNum::January))
    );
}

#[test]
fn test_get_first_month_with_balance() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 3_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 200);
    assert_eq!(
        test_struct.get_first_month_with_balance(),
        Some((2022, MonthNum::January))
    );
}

#[test]
fn test_get_last_month() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 3_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 200);
    assert_eq!(
        test_struct.get_last_month(),
        Some((2022, MonthNum::December))
    );
}

#[test]
fn test_get_last_month_with_balance() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 3_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 200);
    assert_eq!(
        test_struct.get_last_month_with_balance(),
        Some((2022, MonthNum::March))
    );
}

#[test]
fn test_month_has_balance() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    assert_eq!(test_struct.month_has_balance(2022, MonthNum::January), true);
    assert_eq!(
        test_struct.month_has_balance(2022, MonthNum::February),
        false
    );
}

#[test]
fn test_has_year() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    assert_eq!(test_struct.has_year(2022), true);
    assert_eq!(test_struct.has_year(2023), false);
}

#[test]
fn test_get_first_year() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2023, 1_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 200);
    assert_eq!(test_struct.get_first_year(), Some(2022));
}

#[test]
fn test_get_first_year_balance() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    let balance_map: BTreeMap<MonthNum, Option<i64>> = [
        (1_i16.try_into().unwrap(), Some(100)),
        (2_i16.try_into().unwrap(), Some(200)),
    ]
    .iter()
    .cloned()
    .collect();
    test_struct.insert_balance_for_year(2022, balance_map.clone());
    test_struct.insert_balance(2023, 1_i16.try_into().unwrap(), 100);
    assert_eq!(test_struct.get_first_year_balance(), Some(balance_map));
}

#[test]
fn test_get_last_year() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2023, 1_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 200);
    assert_eq!(test_struct.get_last_year(), Some(2023));
}

#[test]
fn test_get_last_year_balance() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    let balance_map: BTreeMap<MonthNum, Option<i64>> = [
        (1_i16.try_into().unwrap(), Some(100)),
        (2_i16.try_into().unwrap(), Some(200)),
    ]
    .iter()
    .cloned()
    .collect();
    test_struct.insert_balance_for_year(2022, balance_map.clone());
    test_struct.insert_balance(2021, 1_i16.try_into().unwrap(), 100);
    assert_eq!(test_struct.get_last_year_balance(), Some(balance_map));
}

#[test]
fn test_is_empty() {
    let test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    assert_eq!(test_struct.is_empty(), true);
}

#[test]
fn test_is_year_empty() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    println!("{test_struct:#?}");
    assert_eq!(test_struct.is_year_empty(2022), false);
    assert_eq!(test_struct.is_year_empty(2023), true);
}

#[test]
fn test_clear_balances() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2023, 2_i16.try_into().unwrap(), 200);
    test_struct.clear_balances(2022);
    assert_eq!(test_struct.get_balance_for_year(2022), None);
    assert_ne!(test_struct.balances().len(), 0);
}

#[test]
fn test_clear_all_balances() {
    let mut test_struct = TestStruct {
        balances: BTreeMap::new(),
    };
    test_struct.insert_balance(2022, 1_i16.try_into().unwrap(), 100);
    test_struct.insert_balance(2023, 2_i16.try_into().unwrap(), 200);
    test_struct.clear_all_balances();
    assert_eq!(test_struct.balances().len(), 0);
}
