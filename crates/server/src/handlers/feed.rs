use axum::extract::{Path, Query, State};
use axum::Json;
use sqlx::PgPool;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_feed::{handlers, models};

use crate::state::AppState;

/// Verify that a profile belongs to the given user — either their own user
/// profile or an agent profile for an agent they own.
async fn verify_profile_ownership(
    pool: &PgPool,
    profile_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let owned: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM profiles p
            LEFT JOIN agents a ON p.profile_type = 'agent' AND p.agent_id = a.id
            WHERE p.id = $1
            AND (
                (p.profile_type = 'user' AND p.user_id = $2)
                OR (p.profile_type = 'agent' AND a.user_id = $2)
            )
        )
        "#,
    )
    .bind(profile_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if !owned {
        return Err(AppError::Forbidden(
            "You do not have permission to post as this profile".into(),
        ));
    }
    Ok(())
}

pub async fn get_feed(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<models::FeedQuery>,
) -> Result<Json<Vec<models::ActivityEvent>>, AppError> {
    let user = super::resolve_user(&state.pool, &auth).await?;
    let events = handlers::get_feed(
        &state.pool,
        user.id,
        query.filter.as_deref(),
        query.limit(),
        query.offset(),
    )
    .await?;
    Ok(Json(events))
}

pub async fn get_profile_activity(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Query(query): Query<aura_network_core::PaginationParams>,
) -> Result<Json<Vec<models::ActivityEvent>>, AppError> {
    let events =
        handlers::get_profile_activity(&state.pool, profile_id, query.limit(), query.offset())
            .await?;
    Ok(Json(events))
}

pub async fn get_post(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<models::ActivityEvent>, AppError> {
    let event = handlers::get_post(&state.pool, post_id).await?;
    Ok(Json(event))
}

pub async fn post_activity(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::CreateActivityEventRequest>,
) -> Result<Json<models::ActivityEvent>, AppError> {
    let user = super::resolve_user(&state.pool, &auth).await?;
    verify_profile_ownership(&state.pool, input.profile_id, user.id).await?;
    let event = handlers::post_activity(&state.pool, input).await?;

    // Broadcast to WebSocket clients
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
    let user = super::resolve_user(&state.pool, &auth).await?;
    let profile = aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await?;
    let comment = handlers::create_comment(&state.pool, event_id, profile.id, input).await?;
    Ok(Json(comment))
}

pub async fn delete_comment(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(comment_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let user = super::resolve_user(&state.pool, &auth).await?;
    let profile = aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await?;
    handlers::delete_comment(&state.pool, comment_id, profile.id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
