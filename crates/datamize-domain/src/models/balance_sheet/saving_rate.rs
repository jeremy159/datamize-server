use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::TransactionDetail;

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SavingRate {
    pub id: Uuid,
    /// Name of the Budgeter
    pub name: String,
    /// The year of the SavingRate, in format 2015.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "1000..3000"))]
    pub year: i32,
    /// Épargne (REER, CELI, compte non enregistré, capital sur le REEE)
    /// achetée avec le revenu net (n’inclue pas le régime de retraite de l’employeur)
    pub savings: Savings,
    /// Pension ou contributions de l’employeur au régime de retraite (REER ou autre)
    /// – inclus au numérateur et dénominateur
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub employer_contribution: i64,
    /// Cotisations au régime de retraite – inclus aux numérateur et dénominateur
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub employee_contribution: i64,
    /// Capital remboursé sur l'hypothèque
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub mortgage_capital: i64,
    /// Revenus nets, soit le montant déposé dans votre compte de banque après toutes les déductions
    /// (impôts, cotisations au régime de retraite, assurance emploi, assurances collectives, RQAP, RRQ)
    /// ainsi que les autres sources de revenus (paies, bonus le cas échéant, RQAP le cas échéant,
    /// allocations pour enfants, remboursement d’impôt, ristourne,
    /// remises en argent de carte de crédit, cadeaux en argent, revenus de location, etc.)
    pub incomes: Incomes,
    // rate = (savings + employer_contribution + employee_contribution + mortgage_capital) / (incomes + employer_contribution + employee_contribution)
    // pub rate: f32, // Leave it to be computed on the frontend
}

impl SavingRate {
    pub fn compute_totals(&mut self, transactions: &[TransactionDetail]) {
        self.savings.compute_total(transactions);
        self.incomes.compute_total(transactions);
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Savings {
    /// Any categories that should be used to compute the savings.
    /// Will take all transactions of the category for the current year.
    pub category_ids: Vec<Uuid>,
    /// Any extra balance to be used, will be added to the total of categories included with this saving.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub extra_balance: i64,
    /// Total balance computed from all categories and extra_balance.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub total: i64,
}

impl Savings {
    pub(crate) fn compute_total(&mut self, transactions: &[TransactionDetail]) {
        let cat_total: i64 = transactions
            .iter()
            .filter(|t| match &t.base.category_id {
                Some(ref id) => self.category_ids.contains(id),
                None => false,
            })
            .map(|t| t.base.amount)
            .sum();

        self.total = cat_total + self.extra_balance;
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Incomes {
    /// Any payees that should be used to compute the incomes.
    /// Will take all transactions of the payee for the current year.
    pub payee_ids: Vec<Uuid>,
    /// Any extra balance to be used, will be added to the total of payees included with this saving.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub extra_balance: i64,
    /// Total balance computed from all categories and extra_balance.
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub total: i64,
}

impl Incomes {
    pub(crate) fn compute_total(&mut self, transactions: &[TransactionDetail]) {
        let cat_total: i64 = transactions
            .iter()
            .filter(|t| match &t.base.payee_id {
                Some(ref id) => self.payee_ids.contains(id),
                None => false,
            })
            .map(|t| t.base.amount)
            .sum();

        self.total = cat_total + self.extra_balance;
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveSavingRate {
    pub id: Uuid,
    pub name: String,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "1000..3000"))]
    pub year: i32,
    pub savings: SaveSavings,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub employer_contribution: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub employee_contribution: i64,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub mortgage_capital: i64,
    pub incomes: SaveIncomes,
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveSavings {
    pub category_ids: Vec<Uuid>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub extra_balance: i64,
}

impl From<SaveSavings> for Savings {
    fn from(value: SaveSavings) -> Self {
        Self {
            category_ids: value.category_ids,
            extra_balance: value.extra_balance,
            total: 0,
        }
    }
}

#[cfg_attr(any(feature = "testutils", test), derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveIncomes {
    pub payee_ids: Vec<Uuid>,
    #[cfg_attr(any(feature = "testutils", test), dummy(faker = "-1000000..1000000"))]
    pub extra_balance: i64,
}

impl From<SaveIncomes> for Incomes {
    fn from(value: SaveIncomes) -> Self {
        Self {
            payee_ids: value.payee_ids,
            extra_balance: value.extra_balance,
            total: 0,
        }
    }
}

impl From<SaveSavingRate> for SavingRate {
    fn from(value: SaveSavingRate) -> Self {
        Self {
            id: value.id,
            name: value.name,
            year: value.year,
            savings: value.savings.into(),
            employer_contribution: value.employer_contribution,
            employee_contribution: value.employee_contribution,
            mortgage_capital: value.mortgage_capital,
            incomes: value.incomes.into(),
        }
    }
}
