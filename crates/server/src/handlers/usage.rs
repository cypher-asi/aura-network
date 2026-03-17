use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_usage::{handlers, models};
use aura_network_users::repo as user_repo;

use crate::state::AppState;

pub async fn get_org_usage(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<models::UsageQuery>,
) -> Result<Json<models::UsageSummary>, AppError> {
    let usage = handlers::get_org_usage(&state.pool, org_id, query.period.as_deref()).await?;
    Ok(Json(usage))
}

pub async fn get_member_usage(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<models::UsageQuery>,
) -> Result<Json<Vec<models::MemberUsage>>, AppError> {
    let usage = handlers::get_member_usage(&state.pool, org_id, query.period.as_deref()).await?;
    Ok(Json(usage))
}

pub async fn get_personal_usage(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<models::UsageQuery>,
) -> Result<Json<models::UsageSummary>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let usage = handlers::get_personal_usage(&state.pool, user.id, query.period.as_deref()).await?;
    Ok(Json(usage))
}

pub async fn get_stats(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Option<models::PlatformStats>>, AppError> {
    let stats = handlers::get_platform_stats(&state.pool).await?;
    Ok(Json(stats))
}
