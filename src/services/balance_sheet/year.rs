use async_trait::async_trait;

use crate::{
    db::balance_sheet::YearRepo,
    error::{AppError, DatamizeResult},
    models::balance_sheet::{SaveYear, UpdateYear, YearDetail, YearSummary},
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait YearServiceExt {
    async fn get_all_years(&self) -> DatamizeResult<Vec<YearSummary>>;
    async fn create_year(&self, new_year: SaveYear) -> DatamizeResult<YearDetail>;
    async fn get_year(&self, year: i32) -> DatamizeResult<YearDetail>;
    async fn update_year(&self, year: i32, new_year: UpdateYear) -> DatamizeResult<YearDetail>;
    async fn delete_year(&self, year: i32) -> DatamizeResult<YearDetail>;
}

pub struct YearService<YR: YearRepo> {
    pub year_repo: YR,
}

#[async_trait]
impl<YR> YearServiceExt for YearService<YR>
where
    YR: YearRepo + Sync + Send,
{
    async fn get_all_years(&self) -> DatamizeResult<Vec<YearSummary>> {
        self.year_repo.get_years_summary().await
    }

    async fn create_year(&self, new_year: SaveYear) -> DatamizeResult<YearDetail> {
        let Err(AppError::ResourceNotFound) =
            self.year_repo.get_year_data_by_number(new_year.year).await else
        {
            return Err(AppError::YearAlreadyExist);
        };

        let year = YearDetail::new(new_year.year);
        self.year_repo.add(&year).await?;

        self.year_repo.update_net_totals(new_year.year).await?;

        self.year_repo.get(new_year.year).await
    }

    async fn get_year(&self, year: i32) -> DatamizeResult<YearDetail> {
        self.year_repo.get(year).await
    }

    async fn update_year(&self, year: i32, new_year: UpdateYear) -> DatamizeResult<YearDetail> {
        let mut year = self.year_repo.get(year).await?;
        year.update_saving_rates(new_year.saving_rates);

        self.year_repo.update_saving_rates(&year).await?;

        Ok(year)
    }

    async fn delete_year(&self, year: i32) -> DatamizeResult<YearDetail> {
        let year_detail = self.year_repo.get(year).await?;
        self.year_repo.delete(year).await?;

        Ok(year_detail)
    }
}