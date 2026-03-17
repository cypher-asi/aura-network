use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use aura_network_auth::InternalAuth;
use aura_network_core::AppError;

use crate::state::AppState;

pub async fn get_user_by_zero_id(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Path(zero_user_id): Path<String>,
) -> Result<Json<aura_network_users::models::User>, AppError> {
    let user = aura_network_users::handlers::get_user_by_zero_id(&state.pool, &zero_user_id).await?;
    Ok(Json(user))
}

pub async fn post_activity(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Json(input): Json<aura_network_feed::models::CreateActivityEventRequest>,
) -> Result<Json<aura_network_feed::models::ActivityEvent>, AppError> {
    let event = aura_network_feed::handlers::post_activity(&state.pool, input).await?;
    Ok(Json(event))
}

pub async fn record_usage(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Json(input): Json<aura_network_usage::models::RecordUsageRequest>,
) -> Result<(), AppError> {
    aura_network_usage::handlers::record_usage(&state.pool, input).await
}

pub async fn check_budget(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<aura_network_usage::models::BudgetStatus>, AppError> {
    let status = aura_network_usage::handlers::check_budget(&state.pool, org_id, user_id).await?;
    Ok(Json(status))
}
