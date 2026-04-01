use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_integrations::{models, repo};
use aura_network_orgs::repo as org_repo;

use crate::state::AppState;

pub async fn create_integration(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(input): Json<models::CreateIntegrationRequest>,
) -> Result<Json<models::OrgIntegration>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    org_repo::require_role(&state.pool, org_id, user.id, "admin").await?;
    let integration = repo::create(&state.pool, org_id, &input).await?;
    Ok(Json(integration))
}

pub async fn list_integrations(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<models::OrgIntegration>>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    org_repo::require_role(&state.pool, org_id, user.id, "admin").await?;
    let integrations = repo::list(&state.pool, org_id).await?;
    Ok(Json(integrations))
}

pub async fn update_integration(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((org_id, integration_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<models::UpdateIntegrationRequest>,
) -> Result<Json<models::OrgIntegration>, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    org_repo::require_role(&state.pool, org_id, user.id, "admin").await?;
    let integration = repo::update_scoped(&state.pool, integration_id, org_id, &input).await?;
    Ok(Json(integration))
}

pub async fn delete_integration(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((org_id, integration_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let user = super::resolve_user(&state, &auth).await?;
    org_repo::require_role(&state.pool, org_id, user.id, "admin").await?;
    repo::delete_scoped(&state.pool, integration_id, org_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
