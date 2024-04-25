use std::sync::Arc;

use datamize_domain::{
    async_trait,
    db::{DbError, DynYearRepo},
    SaveYear, Year,
};

use crate::error::{AppError, DatamizeResult};

#[async_trait]
pub trait YearServiceExt: Send + Sync {
    async fn get_all_years(&self) -> DatamizeResult<Vec<Year>>;
    async fn get_all_years_num(&self) -> DatamizeResult<Vec<i32>>;
    async fn create_year(&self, new_year: SaveYear) -> DatamizeResult<Year>;
    async fn get_year(&self, year: i32) -> DatamizeResult<Year>;
    async fn delete_year(&self, year: i32) -> DatamizeResult<Year>;
}

pub type DynYearService = Arc<dyn YearServiceExt>;

pub struct YearService {
    pub year_repo: DynYearRepo,
}

impl YearService {
    pub fn new_arced(year_repo: DynYearRepo) -> Arc<Self> {
        Arc::new(Self { year_repo })
    }
}

#[async_trait]
impl YearServiceExt for YearService {
    #[tracing::instrument(skip(self))]
    async fn get_all_years(&self) -> DatamizeResult<Vec<Year>> {
        Ok(self.year_repo.get_years().await?)
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_years_num(&self) -> DatamizeResult<Vec<i32>> {
        Ok(self
            .year_repo
            .get_years_data()
            .await?
            .into_iter()
            .map(|y| y.year)
            .collect())
    }

    // TODO: Create year should also create all months with no resources linked to them.
    #[tracing::instrument(skip_all)]
    async fn create_year(&self, new_year: SaveYear) -> DatamizeResult<Year> {
        let Err(DbError::NotFound) = self.year_repo.get_year_data_by_number(new_year.year).await
        else {
            return Err(AppError::ResourceAlreadyExist);
        };

        let year = Year::new(new_year.year);
        self.year_repo.add(&year).await?;

        self.year_repo.update_net_totals(new_year.year).await?;

        Ok(self.year_repo.get(new_year.year).await?)
    }

    #[tracing::instrument(skip(self))]
    async fn get_year(&self, year: i32) -> DatamizeResult<Year> {
        Ok(self.year_repo.get(year).await?)
    }

    #[tracing::instrument(skip(self))]
    async fn delete_year(&self, year: i32) -> DatamizeResult<Year> {
        let year_detail = self.year_repo.get(year).await?;
        self.year_repo.delete(year).await?;

        Ok(year_detail)
    }
}
