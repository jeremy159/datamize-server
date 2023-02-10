use serde::Serialize;
use uuid::Uuid;

use super::{Month, TotalSummary};

#[derive(Debug, Serialize)]
pub struct YearSummary {
    pub id: Uuid,
    /// The year of the date, in format 2015.
    pub year: i32,
    /// The final total net assets of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_assets: TotalSummary,
    /// The final total net portfolio of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_portfolio: TotalSummary,
}

#[derive(Debug, Serialize)]
pub struct SavingRatesPerPerson {
    pub name: String,
    /// Épargne (REER, CELI, compte non enregistré, capital sur le REEE)
    /// achetée avec le revenu net (n’inclue pas le régime de retraite de l’employeur)
    pub savings: i64,
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
    pub incomes: i64,
    /// rate = (savings + employer_contribution + employee_contribution + mortgage_capital) / (incomes + employer_contribution + employee_contribution)
    pub rate: f64,
}

#[derive(Debug, Serialize)]
pub struct YearDetail {
    pub id: Uuid,
    /// The year of the date, in format 2015.
    pub year: i32,
    /// The final total net assets of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_assets: TotalSummary,
    /// The final total net portfolio of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_portfolio: TotalSummary,
    /// All the months of the year.
    pub months: Vec<Month>,
    /// The common saving rates of the year.
    pub saving_rates: Vec<SavingRatesPerPerson>,
}
