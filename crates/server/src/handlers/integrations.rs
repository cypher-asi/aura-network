use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_integrations::{models, repo};

use crate::state::AppState;

pub async fn create_integration(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(input): Json<models::CreateIntegrationRequest>,
) -> Result<Json<models::OrgIntegration>, AppError> {
    let integration = repo::create(&state.pool, org_id, &input).await?;
    Ok(Json(integration))
}

pub async fn list_integrations(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<models::OrgIntegration>>, AppError> {
    let integrations = repo::list(&state.pool, org_id).await?;
    Ok(Json(integrations))
}

pub async fn update_integration(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path((_org_id, integration_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<models::UpdateIntegrationRequest>,
) -> Result<Json<models::OrgIntegration>, AppError> {
    let integration = repo::update(&state.pool, integration_id, &input).await?;
    Ok(Json(integration))
}

pub async fn delete_integration(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path((_org_id, integration_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    repo::delete(&state.pool, integration_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
