use axum::extract::{Path, Query, State};
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_orgs::repo as org_repo;
use aura_network_usage::{handlers, models};

use super::resolve_user;
use crate::state::AppState;

pub async fn get_org_usage(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<models::UsageQuery>,
) -> Result<Json<models::UsageSummary>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
    org_repo::get_member(&state.pool, org_id, user.id).await?;
    let usage = handlers::get_org_usage(&state.pool, org_id, query.period.as_deref()).await?;
    Ok(Json(usage))
}

pub async fn get_member_usage(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(query): Query<models::UsageQuery>,
) -> Result<Json<Vec<models::MemberUsage>>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
    org_repo::require_role(&state.pool, org_id, user.id, "admin").await?;
    let usage = handlers::get_member_usage(&state.pool, org_id, query.period.as_deref()).await?;
    Ok(Json(usage))
}

pub async fn get_personal_usage(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<models::UsageQuery>,
) -> Result<Json<models::UsageSummary>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
    let usage = handlers::get_personal_usage(&state.pool, user.id, query.period.as_deref()).await?;
    Ok(Json(usage))
}

pub async fn record_usage(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::RecordUsageRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    let _user = resolve_user(&state.pool, &auth).await?;
    handlers::record_usage(&state.pool, input).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn get_stats(
    _auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<models::RealTimePlatformStats>, AppError> {
    let stats = handlers::get_realtime_platform_stats(&state.pool).await?;
    Ok(Json(stats))
}
