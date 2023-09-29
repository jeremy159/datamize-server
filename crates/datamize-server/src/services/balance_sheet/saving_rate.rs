use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
use futures::{stream::FuturesUnordered, StreamExt};
use uuid::Uuid;
use ynab::TransactionDetail;

use crate::{
    db::balance_sheet::DynSavingRateRepo,
    error::{AppError, DatamizeResult},
    models::balance_sheet::{SaveSavingRate, SavingRate},
    services::budget_providers::DynTransactionService,
};

#[async_trait]
pub trait SavingRateServiceExt: DynClone + Send + Sync {
    async fn get_all_from_year(&mut self, year: i32) -> DatamizeResult<Vec<SavingRate>>;
    async fn get_saving_rate(&mut self, saving_rate_id: Uuid) -> DatamizeResult<SavingRate>;
    async fn create_saving_rate(
        &mut self,
        new_saving_rate: SaveSavingRate,
    ) -> DatamizeResult<SavingRate>;
    async fn update_saving_rate(
        &mut self,
        new_saving_rate: SavingRate,
    ) -> DatamizeResult<SavingRate>;
    async fn delete_saving_rate(&mut self, saving_rate_id: Uuid) -> DatamizeResult<SavingRate>;
}

clone_trait_object!(SavingRateServiceExt);

pub type DynSavingRateService = Box<dyn SavingRateServiceExt>;

#[derive(Clone)]
pub struct SavingRateService {
    pub saving_rate_repo: DynSavingRateRepo,
    pub transaction_service: DynTransactionService,
}

#[async_trait]
impl SavingRateServiceExt for SavingRateService {
    #[tracing::instrument(skip(self))]
    async fn get_all_from_year(&mut self, year: i32) -> DatamizeResult<Vec<SavingRate>> {
        let mut saving_rates = self.saving_rate_repo.get_from_year(year).await?;
        self.transaction_service
            .refresh_saved_transactions()
            .await?;

        for saving_rate in &mut saving_rates {
            let transactions = self.get_transactions_for(saving_rate).await;

            saving_rate.compute_totals(&transactions);
        }

        Ok(saving_rates)
    }

    #[tracing::instrument(skip(self))]
    async fn get_saving_rate(&mut self, saving_rate_id: Uuid) -> DatamizeResult<SavingRate> {
        let mut saving_rate = self.saving_rate_repo.get(saving_rate_id).await?;
        self.transaction_service
            .refresh_saved_transactions()
            .await?;
        let transactions = self.get_transactions_for(&saving_rate).await;

        saving_rate.compute_totals(&transactions);

        Ok(saving_rate)
    }

    #[tracing::instrument(skip_all)]
    async fn create_saving_rate(
        &mut self,
        new_saving_rate: SaveSavingRate,
    ) -> DatamizeResult<SavingRate> {
        let Err(AppError::ResourceNotFound) = self
            .saving_rate_repo
            .get_by_name(&new_saving_rate.name)
            .await
        else {
            return Err(AppError::ResourceAlreadyExist);
        };

        let mut saving_rate: SavingRate = new_saving_rate.into();
        self.saving_rate_repo.update(&saving_rate).await?;

        self.transaction_service
            .refresh_saved_transactions()
            .await?;
        let transactions = self.get_transactions_for(&saving_rate).await;

        saving_rate.compute_totals(&transactions);

        Ok(saving_rate)
    }

    async fn update_saving_rate(
        &mut self,
        new_saving_rate: SavingRate,
    ) -> DatamizeResult<SavingRate> {
        let Ok(_) = self.saving_rate_repo.get(new_saving_rate.id).await else {
            return Err(AppError::ResourceNotFound);
        };

        self.saving_rate_repo.update(&new_saving_rate).await?;

        let mut saving_rate = new_saving_rate;

        self.transaction_service
            .refresh_saved_transactions()
            .await?;
        let transactions = self.get_transactions_for(&saving_rate).await;

        saving_rate.compute_totals(&transactions);

        Ok(saving_rate)
    }

    #[tracing::instrument(skip(self))]
    async fn delete_saving_rate(&mut self, saving_rate_id: Uuid) -> DatamizeResult<SavingRate> {
        let mut saving_rate = self.saving_rate_repo.get(saving_rate_id).await?;
        self.saving_rate_repo.delete(saving_rate_id).await?;

        self.transaction_service
            .refresh_saved_transactions()
            .await?;
        let transactions = self.get_transactions_for(&saving_rate).await;

        saving_rate.compute_totals(&transactions);

        Ok(saving_rate)
    }
}

impl SavingRateService {
    pub fn new_boxed(
        saving_rate_repo: DynSavingRateRepo,
        transaction_service: DynTransactionService,
    ) -> Box<Self> {
        Box::new(Self {
            saving_rate_repo,
            transaction_service,
        })
    }

    async fn get_transactions_for(&mut self, saving_rate: &SavingRate) -> Vec<TransactionDetail> {
        let transactions = {
            let mut t1 = saving_rate
                .savings
                .category_ids
                .iter()
                .map(|cat_id| {
                    self.transaction_service
                        .get_transactions_by_category_id(*cat_id)
                })
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;

            let t2 = saving_rate
                .incomes
                .payee_ids
                .iter()
                .map(|payee_id| {
                    self.transaction_service
                        .get_transactions_by_payee_id(*payee_id)
                })
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;

            t1.extend(t2);

            t1
        };

        transactions
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<_>>()
    }
}
