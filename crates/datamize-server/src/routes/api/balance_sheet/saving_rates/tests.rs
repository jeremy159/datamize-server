use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Datelike, NaiveDate};
use datamize_domain::SavingRate;
use fake::{faker::chrono::en::Date, Dummy, Fake, Faker};
use tower::ServiceExt;

use crate::{
    error::{AppError, DatamizeResult},
    routes::api::balance_sheet::{get_saving_rate_routes, testutils::saving_rate_service},
};

#[track_caller]
async fn check_get_all(
    input: i32,
    expected_status: StatusCode,
    expected_resp: DatamizeResult<Vec<SavingRate>>,
) {
    let (
        saving_rate_service,
        mut saving_rate_repo,
        mut ynab_transaction_repo,
        mut ynab_transaction_meta_repo,
        mut ynab_client,
    ) = saving_rate_service();

    // saving_rate_repo
    //     .expect_get_from_year()
    //     .returning(|_| expected_resp);

    let app = get_saving_rate_routes(saving_rate_service);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/years/{:?}/saving_rates", input))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), expected_status);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if let Ok(expected_resp) = expected_resp {
        let body: Vec<SavingRate> = serde_json::from_slice(&body).unwrap();
        assert_eq!(body, expected_resp);
    }
}

// #[tokio::test]
async fn balance_sheet_saving_rates_success() {
    check_get_all(
        Date().fake::<NaiveDate>().year(),
        StatusCode::OK,
        Ok(Faker.fake()),
    )
    .await;
}

// #[tokio::test]
async fn balance_sheet_saving_rates_error_500() {
    check_get_all(
        Date().fake::<NaiveDate>().year(),
        StatusCode::INTERNAL_SERVER_ERROR,
        Err(AppError::InternalServerError(Into::into(
            // FIXME: This is not clonable, so maybe not working at the moment
            sqlx::Error::RowNotFound,
        ))),
    )
    .await;
}
