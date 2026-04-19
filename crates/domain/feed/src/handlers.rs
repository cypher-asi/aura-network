use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{
    ActivityEvent, Comment, CreateActivityEventRequest, CreateCommentRequest, PublicFeedbackEntry,
    VoteSummary,
};
use crate::repo;

pub use crate::repo::FeedSort;

pub async fn post_activity(
    pool: &PgPool,
    input: CreateActivityEventRequest,
) -> Result<ActivityEvent, AppError> {
    repo::post_activity(pool, &input).await
}

pub async fn get_feed(
    pool: &PgPool,
    user_id: Uuid,
    viewer_profile_id: Option<Uuid>,
    filter: Option<&str>,
    sort: FeedSort,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    repo::get_feed(
        pool,
        user_id,
        viewer_profile_id,
        filter,
        sort,
        limit,
        offset,
    )
    .await
}

pub async fn get_profile_activity(
    pool: &PgPool,
    profile_id: Uuid,
    viewer_profile_id: Option<Uuid>,
    viewer_user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    repo::get_profile_activity(
        pool,
        profile_id,
        viewer_profile_id,
        viewer_user_id,
        limit,
        offset,
    )
    .await
}

pub async fn get_post(
    pool: &PgPool,
    post_id: Uuid,
    viewer_profile_id: Option<Uuid>,
) -> Result<ActivityEvent, AppError> {
    repo::get_post_by_id(pool, post_id, viewer_profile_id).await
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

/// Validate a vote string and convert it to the persisted SMALLINT value.
/// Returns `None` when the viewer is clearing their vote.
pub fn parse_vote(raw: &str) -> Result<Option<i16>, AppError> {
    match raw {
        "up" => Ok(Some(1)),
        "down" => Ok(Some(-1)),
        "none" => Ok(None),
        other => Err(AppError::BadRequest(format!("invalid vote: '{other}'"))),
    }
}

pub async fn cast_vote(
    pool: &PgPool,
    post_id: Uuid,
    profile_id: Uuid,
    raw_vote: &str,
) -> Result<VoteSummary, AppError> {
    let value = parse_vote(raw_vote)?;
    repo::upsert_vote(pool, post_id, profile_id, value).await
}

pub async fn get_vote_summary(
    pool: &PgPool,
    post_id: Uuid,
    viewer_profile_id: Option<Uuid>,
) -> Result<VoteSummary, AppError> {
    repo::get_vote_summary(pool, post_id, viewer_profile_id).await
}

pub async fn list_public_feedback(
    pool: &PgPool,
    product: &str,
    sort: Option<&str>,
    category: Option<&str>,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<PublicFeedbackEntry>, AppError> {
    repo::list_public_feedback(pool, product, sort, category, status, limit).await
}

pub async fn patch_post_metadata(
    pool: &PgPool,
    post_id: Uuid,
    viewer_profile_id: Option<Uuid>,
    patch: &serde_json::Value,
) -> Result<ActivityEvent, AppError> {
    repo::patch_metadata(pool, post_id, viewer_profile_id, patch).await
}
