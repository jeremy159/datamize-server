#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ynab(ynab::Error),
}
