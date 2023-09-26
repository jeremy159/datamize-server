use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ynab::TransactionDetail;

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SavingRate {
    pub id: Uuid,
    /// Name of the Budgeter
    pub name: String,
    /// The year of the SavingRate, in format 2015.
    pub year: i32,
    /// Épargne (REER, CELI, compte non enregistré, capital sur le REEE)
    /// achetée avec le revenu net (n’inclue pas le régime de retraite de l’employeur)
    pub savings: Savings,
    /// Pension ou contributions de l’employeur au régime de retraite (REER ou autre)
    /// – inclus au numérateur et dénominateur
    pub employer_contribution: i64,
    /// Cotisations au régime de retraite – inclus aux numérateur et dénominateur
    pub employee_contribution: i64,
    /// Capital remboursé sur l'hypothèque
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

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Savings {
    /// Any categories that should be used to compute the savings.
    /// Will take all transactions of the category for the current year.
    pub category_ids: Vec<Uuid>, // TODO: to use /budgets/{budget_id}/categories/{category_id}/transactions
    /// Any extra balance to be used, will be added to the total of categories included with this saving.
    pub extra_balance: i64,
    /// Total balance computed from all categories and extra_balance.
    pub total: i64,
}

impl Savings {
    fn compute_total(&mut self, transactions: &[TransactionDetail]) {
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

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Incomes {
    /// Any payees that should be used to compute the incomes.
    /// Will take all transactions of the payee for the current year.
    pub payee_ids: Vec<Uuid>, // TODO: to use /budgets/{budget_id}/payees/{payee_id}/transactions
    /// Any extra balance to be used, will be added to the total of payees included with this saving.
    pub extra_balance: i64,
    /// Total balance computed from all categories and extra_balance.
    pub total: i64,
}

impl Incomes {
    fn compute_total(&mut self, transactions: &[TransactionDetail]) {
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

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveSavingRate {
    pub id: Uuid,
    pub name: String,
    pub year: i32,
    pub savings: SaveSavings,
    pub employer_contribution: i64,
    pub employee_contribution: i64,
    pub mortgage_capital: i64,
    pub incomes: SaveIncomes,
}

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveSavings {
    pub category_ids: Vec<Uuid>,
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

#[cfg_attr(test, derive(fake::Dummy))]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveIncomes {
    pub payee_ids: Vec<Uuid>,
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
            id: Uuid::new_v4(),
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
