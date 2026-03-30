use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{Profile, UpdateUserRequest, User};
use crate::repo;

/// Get current authenticated user. Caller must provide the zero_user_id
/// (extracted from JWT by the server layer).
pub async fn get_me(pool: &PgPool, zero_user_id: &str) -> Result<User, AppError> {
    repo::get_by_zero_id(pool, zero_user_id).await
}

/// Update current user's profile fields.
pub async fn update_me(
    pool: &PgPool,
    zero_user_id: &str,
    input: UpdateUserRequest,
) -> Result<User, AppError> {
    let existing = repo::get_by_zero_id(pool, zero_user_id).await?;
    repo::update(pool, existing.id, &input).await
}

/// Get a user by their internal UUID.
pub async fn get_user(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    repo::get_by_id(pool, user_id).await
}

/// Get a profile by ID (could be user or agent profile).
pub async fn get_profile(pool: &PgPool, profile_id: Uuid) -> Result<Profile, AppError> {
    repo::get_profile(pool, profile_id).await
}

/// Get a user by their Zero platform ID (for internal service lookups).
pub async fn get_user_by_zero_id(pool: &PgPool, zero_user_id: &str) -> Result<User, AppError> {
    repo::get_by_zero_id(pool, zero_user_id).await
}
