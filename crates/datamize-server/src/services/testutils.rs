use crate::error::AppError;

pub(crate) enum ErrorType {
    Internal,
    NotFound,
    AlreadyExist,
}

pub(crate) fn assert_err(err: AppError, expected_err: Option<ErrorType>) {
    match expected_err {
        Some(ErrorType::Internal) => assert!(matches!(err, AppError::InternalServerError(_))),
        Some(ErrorType::NotFound) => {
            assert!(matches!(err, AppError::ResourceNotFound))
        }
        Some(ErrorType::AlreadyExist) => assert!(matches!(err, AppError::ResourceAlreadyExist)),
        None => unreachable!(),
    }
}
