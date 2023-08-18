use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    db::balance_sheet::{MonthRepo, PostgresMonthRepo},
    error::{AppError, DatamizeResult},
    models::balance_sheet::{Month, MonthNum, SaveMonth},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MonthServiceExt {
    async fn get_all_months(&self) -> DatamizeResult<Vec<Month>>;
    async fn get_all_months_from_year(&self, year: i32) -> DatamizeResult<Vec<Month>>;
    async fn create_month(&self, year: i32, new_month: SaveMonth) -> DatamizeResult<Month>;
    async fn get_month(&self, month: MonthNum, year: i32) -> DatamizeResult<Month>;
    async fn delete_month(&self, month: MonthNum, year: i32) -> DatamizeResult<Month>;
}

pub type DynMonthService = Arc<dyn MonthServiceExt + Send + Sync>;

pub struct MonthService {
    pub month_repo: Arc<dyn MonthRepo + Sync + Send>,
}

#[async_trait]
impl MonthServiceExt for MonthService {
    #[tracing::instrument(skip(self))]
    async fn get_all_months(&self) -> DatamizeResult<Vec<Month>> {
        self.month_repo.get_months().await
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_months_from_year(&self, year: i32) -> DatamizeResult<Vec<Month>> {
        self.month_repo.get_months_of_year(year).await
    }

    #[tracing::instrument(skip(self, new_month))]
    async fn create_month(&self, year: i32, new_month: SaveMonth) -> DatamizeResult<Month> {
        self.month_repo.get_year_data_by_number(year).await?;

        let Err(AppError::ResourceNotFound) =
            self.month_repo.get_month_data_by_number(new_month.month, year).await else
        {
            return Err(AppError::MonthAlreadyExist);
        };

        let month = Month::new(new_month.month, year);
        self.month_repo.add(&month, year).await?;

        self.month_repo
            .update_net_totals(new_month.month, year)
            .await?;

        self.month_repo.get(new_month.month, year).await
    }

    #[tracing::instrument(skip(self))]
    async fn get_month(&self, month: MonthNum, year: i32) -> DatamizeResult<Month> {
        self.month_repo.get(month, year).await
    }

    #[tracing::instrument(skip(self))]
    async fn delete_month(&self, month: MonthNum, year: i32) -> DatamizeResult<Month> {
        let month_detail = self.month_repo.get(month, year).await?;
        self.month_repo.delete(month, year).await?;

        Ok(month_detail)
    }
}

impl MonthService {
    pub fn new_arced(db_conn_pool: PgPool) -> Arc<Self> {
        Arc::new(Self {
            month_repo: PostgresMonthRepo::new_arced(db_conn_pool),
        })
    }
}
