use super::types::{
    BugdetCalculationDataConfig, FixedExpenses, LongTermSavings, PersonSalaryConfig,
    RetirementSavings, ShortTermSavings, VariableExpenses,
};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BudgetDataConfig {
    pub budget_calculation_data: BugdetCalculationDataConfig,
    pub person_salaries: Vec<PersonSalaryConfig>,
}

impl Default for BudgetDataConfig {
    fn default() -> BudgetDataConfig {
        BudgetDataConfig {
            budget_calculation_data: BugdetCalculationDataConfig {
                fixed_expenses: FixedExpenses {
                    housing_ids: vec![],
                    transport_ids: vec![],
                    other_ids: vec![],
                },
                variable_expenses: VariableExpenses {
                    subscription_ids: vec![],
                    other_ids: vec![],
                },
                short_term_savings: ShortTermSavings { ids: vec![] },
                long_term_savings: LongTermSavings { ids: vec![] },
                retirement_savings: RetirementSavings { ids: vec![] },
                external_expenses: vec![],
            },
            person_salaries: vec![],
        }
    }
}

impl BudgetDataConfig {
    pub fn build() -> Self {
        Figment::from(Serialized::defaults(BudgetDataConfig::default()))
            .merge(Toml::file(
                std::env::current_dir()
                    .unwrap()
                    .join("budget-data-api/budget-data-config.toml"),
            ))
            .merge(Env::prefixed("DATAMIZE_"))
            .extract()
            .expect("Failed to extract config files and environment variables into a rust struct.")
    }
}
