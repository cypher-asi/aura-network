use axum::extract::State;
use axum::Json;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_users::{models, repo};

use crate::state::AppState;

pub async fn redeem_code(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::RedeemAccessCodeRequest>,
) -> Result<Json<models::AppAccessCode>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    let code = repo::redeem_access_code(&state.pool, &input.code, user.id).await?;
    Ok(Json(code))
}

/// Grant access to the current user. Called by aura-os when a Pro user
/// logs in and doesn't have is_access_granted set yet.
pub async fn grant_access(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<axum::http::StatusCode, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    if !user.is_access_granted {
        repo::grant_access(&state.pool, user.id).await?;
        repo::ensure_access_code(&state.pool, user.id).await?;
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

/// Returns the user's single access code. Auto-generates one if the user
/// has access but no code exists yet.
pub async fn get_my_code(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Option<models::AppAccessCode>>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;

    if user.is_access_granted {
        let code = repo::ensure_access_code(&state.pool, user.id).await?;
        return Ok(Json(Some(code)));
    }

    let code = repo::get_access_code(&state.pool, user.id).await?;
    Ok(Json(code))
}
