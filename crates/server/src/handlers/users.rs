use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_users::{handlers, models};

use crate::state::AppState;
use super::resolve_user;

pub async fn get_me(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<models::User>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
    Ok(Json(user))
}

pub async fn update_me(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::UpdateUserRequest>,
) -> Result<Json<models::User>, AppError> {
    let _existing = resolve_user(&state.pool, &auth).await?;
    let user = handlers::update_me(&state.pool, &auth.user_id, input).await?;
    Ok(Json(user))
}

pub async fn get_user(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<models::User>, AppError> {
    let user = handlers::get_user(&state.pool, user_id).await?;
    Ok(Json(user))
}

pub async fn get_profile(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<models::Profile>, AppError> {
    let profile = handlers::get_profile(&state.pool, profile_id).await?;
    Ok(Json(profile))
}
