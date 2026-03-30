use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_users::{handlers, models, repo};

use super::resolve_user;
use crate::state::AppState;

async fn user_with_profile(pool: &sqlx::PgPool, user: models::User) -> models::UserResponse {
    let profile_id = repo::get_profile_by_user_id(pool, user.id)
        .await
        .ok()
        .map(|p| p.id);
    models::UserResponse { user, profile_id }
}

pub async fn get_me(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<models::UserResponse>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
    Ok(Json(user_with_profile(&state.pool, user).await))
}

pub async fn update_me(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::UpdateUserRequest>,
) -> Result<Json<models::UserResponse>, AppError> {
    let _existing = resolve_user(&state.pool, &auth).await?;
    let user = handlers::update_me(&state.pool, &auth.user_id, input).await?;
    Ok(Json(user_with_profile(&state.pool, user).await))
}

pub async fn get_user(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<models::UserResponse>, AppError> {
    let user = handlers::get_user(&state.pool, user_id).await?;
    Ok(Json(user_with_profile(&state.pool, user).await))
}

pub async fn get_profile(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<models::Profile>, AppError> {
    let profile = handlers::get_profile(&state.pool, profile_id).await?;
    Ok(Json(profile))
}

pub async fn get_user_profile(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<models::Profile>, AppError> {
    let profile = repo::get_profile_by_user_id(&state.pool, user_id).await?;
    Ok(Json(profile))
}

pub async fn get_agent_profile(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<models::Profile>, AppError> {
    let profile = repo::get_profile_by_agent_id(&state.pool, agent_id).await?;
    Ok(Json(profile))
}
