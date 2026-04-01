use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_social::{handlers, models};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub period: Option<String>,
    pub org_id: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn follow(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::FollowRequest>,
) -> Result<Json<models::Follow>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    let profile = aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await?;
    let follow = handlers::follow(&state.pool, profile.id, input.target_profile_id).await?;
    Ok(Json(follow))
}

pub async fn unfollow(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(target_profile_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    let profile = aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await?;
    handlers::unfollow(&state.pool, profile.id, target_profile_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn list_following(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<models::Follow>>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    let profile = aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await?;
    let follows = handlers::list_following(&state.pool, profile.id).await?;
    Ok(Json(follows))
}

pub async fn leaderboard(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<Vec<models::LeaderboardEntry>>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100).max(1);
    let entries =
        handlers::get_leaderboard(&state.pool, query.period.as_deref(), query.org_id, limit)
            .await?;
    Ok(Json(entries))
}
