pub mod errors;
pub mod ids;
pub mod pagination;

pub use errors::AppError;
pub use ids::*;
pub use pagination::PaginationParams;

pub type Result<T> = std::result::Result<T, AppError>;
