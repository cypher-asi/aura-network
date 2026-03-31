use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{ActivityEvent, Comment, CreateActivityEventRequest, CreateCommentRequest};
use crate::repo;

pub async fn post_activity(
    pool: &PgPool,
    input: CreateActivityEventRequest,
) -> Result<ActivityEvent, AppError> {
    repo::post_activity(pool, &input).await
}

pub async fn get_feed(
    pool: &PgPool,
    user_id: Uuid,
    filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    repo::get_feed(pool, user_id, filter, limit, offset).await
}

pub async fn get_profile_activity(
    pool: &PgPool,
    profile_id: Uuid,
    viewer_user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    repo::get_profile_activity(pool, profile_id, viewer_user_id, limit, offset).await
}

pub async fn get_post(pool: &PgPool, post_id: Uuid) -> Result<ActivityEvent, AppError> {
    repo::get_post_by_id(pool, post_id).await
}

pub async fn create_comment(
    pool: &PgPool,
    activity_event_id: Uuid,
    profile_id: Uuid,
    input: CreateCommentRequest,
) -> Result<Comment, AppError> {
    repo::create_comment(pool, activity_event_id, profile_id, &input).await
}

pub async fn list_comments(
    pool: &PgPool,
    activity_event_id: Uuid,
) -> Result<Vec<Comment>, AppError> {
    repo::list_comments(pool, activity_event_id).await
}

pub async fn delete_comment(
    pool: &PgPool,
    comment_id: Uuid,
    profile_id: Uuid,
) -> Result<(), AppError> {
    repo::delete_comment(pool, comment_id, profile_id).await
}
