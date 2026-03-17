use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{Follow, LeaderboardEntry};
use crate::repo;

pub async fn follow(
    pool: &PgPool,
    follower_profile_id: Uuid,
    target_profile_id: Uuid,
) -> Result<Follow, AppError> {
    repo::follow(pool, follower_profile_id, target_profile_id).await
}

pub async fn unfollow(
    pool: &PgPool,
    follower_profile_id: Uuid,
    target_profile_id: Uuid,
) -> Result<(), AppError> {
    repo::unfollow(pool, follower_profile_id, target_profile_id).await
}

pub async fn list_following(
    pool: &PgPool,
    follower_profile_id: Uuid,
) -> Result<Vec<Follow>, AppError> {
    repo::list_following(pool, follower_profile_id).await
}

pub async fn get_leaderboard(
    pool: &PgPool,
    period: Option<&str>,
    org_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, AppError> {
    repo::get_leaderboard(pool, period, org_id, limit).await
}
