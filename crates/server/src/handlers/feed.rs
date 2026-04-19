use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_feed::{handlers, models};

use crate::state::AppState;

/// Best-effort viewer profile lookup. Returns `None` if the viewer has no
/// profile yet (e.g. a freshly-registered user); vote aggregates will then
/// report `viewerVote = "none"` without erroring.
async fn viewer_profile_id(
    state: &AppState,
    auth: &AuthUser,
) -> Result<Option<Uuid>, AppError> {
    let user = super::resolve_user(state, auth).await?;
    match aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await {
        Ok(profile) => Ok(Some(profile.id)),
        Err(AppError::NotFound(_)) => Ok(None),
        Err(err) => Err(err),
    }
}

async fn require_viewer_profile_id(
    state: &AppState,
    auth: &AuthUser,
) -> Result<Uuid, AppError> {
    let user = super::resolve_user(state, auth).await?;
    let profile = aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await?;
    Ok(profile.id)
}

pub async fn get_feed(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<models::FeedQuery>,
) -> Result<Json<Vec<models::ActivityEvent>>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    let viewer_profile_id = match aura_network_users::repo::get_profile_by_user_id(
        &state.pool,
        user.id,
    )
    .await
    {
        Ok(profile) => Some(profile.id),
        Err(AppError::NotFound(_)) => None,
        Err(err) => return Err(err),
    };
    let sort = match query.sort.as_deref() {
        Some(raw) => handlers::FeedSort::from_str(raw).ok_or_else(|| {
            AppError::BadRequest(format!("invalid feed sort: '{raw}'"))
        })?,
        None => handlers::FeedSort::default(),
    };
    let events = handlers::get_feed(
        &state.pool,
        user.id,
        viewer_profile_id,
        query.filter.as_deref(),
        sort,
        query.limit(),
        query.offset(),
    )
    .await?;
    Ok(Json(events))
}

pub async fn get_profile_activity(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Query(query): Query<aura_network_core::PaginationParams>,
) -> Result<Json<Vec<models::ActivityEvent>>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    let viewer = viewer_profile_id(&state, &auth).await?;
    let events = handlers::get_profile_activity(
        &state.pool,
        profile_id,
        viewer,
        user.id,
        query.limit(),
        query.offset(),
    )
    .await?;
    Ok(Json(events))
}

pub async fn get_post(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<models::ActivityEvent>, AppError> {
    let viewer = viewer_profile_id(&state, &auth).await?;
    let event = handlers::get_post(&state.pool, post_id, viewer).await?;
    Ok(Json(event))
}

pub async fn post_activity(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(mut input): Json<models::CreateActivityEventRequest>,
) -> Result<Json<models::ActivityEvent>, AppError> {
    // Fall back to the authenticated user's profile when the client doesn't
    // pass profileId explicitly (thin proxies like aura-os-server don't
    // always carry it in their session).
    if input.profile_id.is_none() {
        input.profile_id = Some(require_viewer_profile_id(&state, &auth).await?);
    } else {
        super::resolve_user(&state, &auth).await?;
    }
    let event = handlers::post_activity(&state.pool, input).await?;

    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "type": "activity.new",
        "data": &event
    })) {
        let _ = state.events_tx.send(json);
    }

    Ok(Json(event))
}

pub async fn list_comments(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<Vec<models::Comment>>, AppError> {
    let comments = handlers::list_comments(&state.pool, event_id).await?;
    Ok(Json(comments))
}

pub async fn create_comment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
    Json(input): Json<models::CreateCommentRequest>,
) -> Result<Json<models::Comment>, AppError> {
    let profile_id = require_viewer_profile_id(&state, &auth).await?;
    let comment = handlers::create_comment(&state.pool, event_id, profile_id, input).await?;
    Ok(Json(comment))
}

pub async fn delete_comment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(comment_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let profile_id = require_viewer_profile_id(&state, &auth).await?;
    handlers::delete_comment(&state.pool, comment_id, profile_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn cast_vote(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Json(input): Json<models::CastVoteRequest>,
) -> Result<Json<models::VoteSummary>, AppError> {
    let profile_id = require_viewer_profile_id(&state, &auth).await?;
    let summary = handlers::cast_vote(&state.pool, post_id, profile_id, &input.vote).await?;
    Ok(Json(summary))
}

pub async fn get_vote_summary(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<models::VoteSummary>, AppError> {
    let viewer = viewer_profile_id(&state, &auth).await?;
    let summary = handlers::get_vote_summary(&state.pool, post_id, viewer).await?;
    Ok(Json(summary))
}

pub async fn patch_post(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Json(input): Json<models::PatchPostRequest>,
) -> Result<Json<models::ActivityEvent>, AppError> {
    let viewer = viewer_profile_id(&state, &auth).await?;
    let metadata = input.metadata.ok_or_else(|| {
        AppError::BadRequest("patch body must include a metadata object".into())
    })?;
    let event = handlers::patch_post_metadata(&state.pool, post_id, viewer, &metadata).await?;
    Ok(Json(event))
}

/// Public, unauthenticated listing of `event_type = 'feedback'` posts. Feeds
/// the marketing-site roadmap page. Deliberately exposes only already-public
/// fields (title, body, vote aggregates, public profile display name) and
/// never surfaces viewer state.
pub async fn list_public_feedback(
    State(state): State<AppState>,
    Query(query): Query<models::PublicFeedbackQuery>,
) -> Result<Json<Vec<models::PublicFeedbackEntry>>, AppError> {
    let entries = handlers::list_public_feedback(
        &state.pool,
        query.product(),
        query.sort.as_deref(),
        query.category.as_deref(),
        query.status.as_deref(),
        query.limit(),
    )
    .await?;
    Ok(Json(entries))
}
