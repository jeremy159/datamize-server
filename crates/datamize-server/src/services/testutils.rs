use datamize_domain::db::DbError;

use crate::error::AppError;

pub(crate) enum ErrorType {
    NotFound,
    AlreadyExist,
    Database,
    // Config,
    ChronoParse,
    // Ynab,
}

pub(crate) fn assert_err(err: AppError, expected_err: Option<ErrorType>) {
    match expected_err {
        Some(ErrorType::NotFound) => {
            assert!(
                matches!(err, AppError::ResourceNotFound)
                    || matches!(err, AppError::DbError(DbError::NotFound))
            )
        }
        Some(ErrorType::AlreadyExist) => assert!(
            matches!(err, AppError::ResourceAlreadyExist)
                || matches!(err, AppError::DbError(DbError::AlreadyExists))
        ),
        Some(ErrorType::Database) => assert!(matches!(err, AppError::DbError(_))),
        // Some(ErrorType::Config) => assert!(matches!(err, AppError::ConfigError(_))),
        Some(ErrorType::ChronoParse) => assert!(matches!(err, AppError::ParseError(_))),
        // Some(ErrorType::Ynab) => assert!(matches!(err, AppError::YnabError(_))),
        None => unreachable!(),
    }
}
