use serde::Serialize;
use uuid::Uuid;

use super::{FinancialResource, NetTotal};

/// A balance sheet of the month.
#[derive(Debug, Serialize)]
pub struct Month {
    pub id: Uuid,
    /// The month in starting with January at 0.
    pub month: chrono::Month,
    /// Net Assets or Net Portfolio summary section. Includes the variation with the previous month.
    pub net_totals: Vec<NetTotal>,
    /// All of the Assets and Liabilities of the current month are regrouped here.
    pub resources: Vec<FinancialResource>,
}
