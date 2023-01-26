use super::types::{
    BugdetCalculationDataConfig, CeliConfig, DatabaseConfig, FixedExpanses, LongTermSavings,
    PersonSalaryConfig, RedisConfig, RetirementSavings, ShortTermSavings, VariableExpanses,
};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BudgetDataConfig {
    pub ynab_pat: String,
    pub ynab_base_url: String,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub budget_calculation_data: BugdetCalculationDataConfig,
    pub person_salaries: Vec<PersonSalaryConfig>,
    pub celis: Vec<CeliConfig>,
}

impl Default for BudgetDataConfig {
    fn default() -> BudgetDataConfig {
        BudgetDataConfig {
            ynab_pat: "".into(),
            ynab_base_url: "https://api.youneedabudget.com/v1/".into(),
            database: DatabaseConfig {
                username: String::from("postgres"),
                password: String::from("password"),
                port: 5432,
                host: String::from("127.0.0.1"),
                database_name: String::from("budget_data"),
            },
            redis: RedisConfig {
                host: String::from("127.0.0.1"),
                port: 6379,
            },
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
            .merge(Toml::file("budget-data-config.toml"))
            .merge(Env::prefixed("BUDGET_DATA_"))
            .extract()
            .expect("Failed to extract config files and environment variables into a rust struct.")
    }
}
