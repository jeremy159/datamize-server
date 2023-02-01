use super::types::{
    BugdetCalculationDataConfig, CeliConfig, FixedExpanses, LongTermSavings, PersonSalaryConfig,
    RetirementSavings, ShortTermSavings, VariableExpanses,
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
    pub celis: Vec<CeliConfig>,
}

impl Default for BudgetDataConfig {
    fn default() -> BudgetDataConfig {
        BudgetDataConfig {
            budget_calculation_data: BugdetCalculationDataConfig {
                fixed_expanses: FixedExpanses {
                    housing_ids: vec![],
                    transport_ids: vec![],
                    other_ids: vec![],
                },
                variable_expanses: VariableExpanses {
                    subscription_ids: vec![],
                    other_ids: vec![],
                },
                short_term_savings: ShortTermSavings { ids: vec![] },
                long_term_savings: LongTermSavings { ids: vec![] },
                retirement_savings: RetirementSavings { ids: vec![] },
                external_expanses: vec![],
            },
            person_salaries: vec![],
            celis: vec![],
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
