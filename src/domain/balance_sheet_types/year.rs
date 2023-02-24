use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Month, NetTotal};

#[derive(Debug, Serialize, Deserialize)]
pub struct YearSummary {
    pub id: Uuid,
    /// The year of the date, in format 2015.
    pub year: i32,
    /// The final total net assets of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_assets: NetTotal,
    /// The final total portfolio of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_portfolio: NetTotal,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SaveYear {
    /// The year of the date, in format 2015.
    pub year: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SavingRatesPerPerson {
    pub id: Uuid,
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
    pub rate: f32,
}

impl SavingRatesPerPerson {
    pub fn new_jeremy() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Taux Épargne Jeremy".to_string(),
            savings: 0,
            employer_contribution: 0,
            employee_contribution: 0,
            mortgage_capital: 0,
            incomes: 0,
            rate: 0.0,
        }
    }

    pub fn new_sandryne() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Taux Épargne Sandryne".to_string(),
            savings: 0,
            employer_contribution: 0,
            employee_contribution: 0,
            mortgage_capital: 0,
            incomes: 0,
            rate: 0.0,
        }
    }

    pub fn new_common() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Taux Épargne Commun".to_string(),
            savings: 0,
            employer_contribution: 0,
            employee_contribution: 0,
            mortgage_capital: 0,
            incomes: 0,
            rate: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YearDetail {
    pub id: Uuid,
    /// The year of the date, in format 2015.
    pub year: i32,
    /// The final total net assets of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_assets: NetTotal,
    /// The final total portfolio of the year.
    /// Basically equals to the total of the year's last month.
    /// The only difference is the variation is calculated with the previous year, not the previous month.
    pub net_portfolio: NetTotal,
    /// All the months of the year.
    pub months: Vec<Month>,
    /// The common saving rates of the year.
    pub saving_rates: Vec<SavingRatesPerPerson>,
}

impl YearDetail {
    pub fn new(year: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            year,
            net_assets: NetTotal::new_asset(),
            net_portfolio: NetTotal::new_portfolio(),
            saving_rates: vec![
                SavingRatesPerPerson::new_jeremy(),
                SavingRatesPerPerson::new_sandryne(),
                SavingRatesPerPerson::new_common(),
            ],
            months: vec![],
        }
    }

    pub fn update_net_assets_with_previous(&mut self, prev_net_assets: &NetTotal) {
        self.net_assets.balance_var = self.net_assets.total - prev_net_assets.total;
        self.net_assets.percent_var =
            self.net_assets.balance_var as f32 / prev_net_assets.total as f32;
    }

    pub fn update_net_portfolio_with_previous(&mut self, prev_net_portfolio: &NetTotal) {
        self.net_portfolio.balance_var = self.net_portfolio.total - prev_net_portfolio.total;
        self.net_portfolio.percent_var =
            self.net_portfolio.balance_var as f32 / prev_net_portfolio.total as f32;
    }

    pub fn get_last_month(&self) -> Option<Month> {
        self.months.last().cloned()
    }

    pub fn needs_net_totals_update(
        &self,
        month_net_assets: &NetTotal,
        month_net_portfolio: &NetTotal,
    ) -> bool {
        self.net_assets.total != month_net_assets.total
            || self.net_portfolio.total != month_net_portfolio.total
    }

    pub fn update_net_assets_with_last_month(&mut self, month_net_assets: &NetTotal) {
        self.net_assets.total = month_net_assets.total;
    }

    pub fn update_net_portfolio_with_last_month(&mut self, month_net_portfolio: &NetTotal) {
        self.net_portfolio.total = month_net_portfolio.total;
    }

    pub fn update_saving_rates(&mut self, saving_rates: Vec<SavingRatesPerPerson>) {
        self.saving_rates = saving_rates;
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateYear {
    pub saving_rates: Vec<SavingRatesPerPerson>,
}
