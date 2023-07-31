use async_trait::async_trait;

use crate::{
    db::balance_sheet::MonthRepo,
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

pub struct MonthService<MR: MonthRepo> {
    pub month_repo: MR,
}

#[async_trait]
impl<MR> MonthServiceExt for MonthService<MR>
where
    MR: MonthRepo + Sync + Send,
{
    async fn get_all_months(&self) -> DatamizeResult<Vec<Month>> {
        self.month_repo.get_months().await
    }

    async fn get_all_months_from_year(&self, year: i32) -> DatamizeResult<Vec<Month>> {
        self.month_repo.get_months_of_year(year).await
    }

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

    async fn get_month(&self, month: MonthNum, year: i32) -> DatamizeResult<Month> {
        self.month_repo.get(month, year).await
    }

    async fn delete_month(&self, month: MonthNum, year: i32) -> DatamizeResult<Month> {
        let month_detail = self.month_repo.get(month, year).await?;
        self.month_repo.delete(month, year).await?;

        Ok(month_detail)
    }
}
