pub mod errors;
#[macro_use]
pub mod ids;
pub mod pagination;

pub use errors::AppError;
pub use pagination::PaginationParams;

pub type Result<T> = std::result::Result<T, AppError>;
