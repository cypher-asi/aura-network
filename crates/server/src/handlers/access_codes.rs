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

pub async fn list_my_codes(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<models::AppAccessCode>>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    let codes = repo::list_access_codes(&state.pool, user.id).await?;
    Ok(Json(codes))
}
