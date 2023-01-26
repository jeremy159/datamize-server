mod client;
mod error;
pub mod types;
pub use client::Client;
pub use error::{ApiError, Error};

pub type Result<T> = std::result::Result<T, Error>;
