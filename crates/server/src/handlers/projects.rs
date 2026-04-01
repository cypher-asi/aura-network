use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_orgs::repo as org_repo;
use aura_network_projects::{handlers, models};

use super::resolve_user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ProjectListQuery {
    pub org_id: Uuid,
}

pub async fn create_project(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::CreateProjectRequest>,
) -> Result<Json<models::Project>, AppError> {
    let user = resolve_user(&state, &auth).await?;
    org_repo::get_member(&state.pool, input.org_id, user.id).await?;
    let project = handlers::create_project(&state.pool, input).await?;
    Ok(Json(project))
}

pub async fn list_projects(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<Vec<models::Project>>, AppError> {
    let user = resolve_user(&state, &auth).await?;
    org_repo::get_member(&state.pool, query.org_id, user.id).await?;
    let projects = handlers::list_projects(&state.pool, query.org_id).await?;
    Ok(Json(projects))
}

pub async fn get_project(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<models::Project>, AppError> {
    let user = resolve_user(&state, &auth).await?;
    let project = handlers::get_project(&state.pool, project_id).await?;
    org_repo::get_member(&state.pool, project.org_id, user.id).await?;
    Ok(Json(project))
}

pub async fn update_project(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<models::UpdateProjectRequest>,
) -> Result<Json<models::Project>, AppError> {
    let user = resolve_user(&state, &auth).await?;
    let existing = handlers::get_project(&state.pool, project_id).await?;
    org_repo::get_member(&state.pool, existing.org_id, user.id).await?;
    let project = handlers::update_project(&state.pool, project_id, input).await?;
    Ok(Json(project))
}

pub async fn delete_project(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user = resolve_user(&state, &auth).await?;
    let existing = handlers::get_project(&state.pool, project_id).await?;
    org_repo::require_role(&state.pool, existing.org_id, user.id, "admin").await?;

    // Check aura-storage for project agents before allowing delete
    if let Some(ref storage_url) = state.aura_storage_url {
        let url = format!(
            "{}/internal/projects/{}/agents/count",
            storage_url, project_id
        );
        let response = reqwest::Client::new()
            .get(&url)
            .header("X-Internal-Token", &state.internal_token.0)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to check project agents in aura-storage");
                AppError::Internal("Failed to verify project agent status".into())
            })?;

        if response.status().is_success() {
            let body: serde_json::Value = response.json().await.map_err(|e| {
                tracing::error!(error = %e, "Failed to parse aura-storage response");
                AppError::Internal("Failed to verify project agent status".into())
            })?;

            let count = body["count"].as_i64().unwrap_or(0);
            if count > 0 {
                return Err(AppError::BadRequest(
                    "Cannot delete project with existing project agents. Delete all project agents first.".into(),
                ));
            }
        }
    }

    handlers::delete_project(&state.pool, project_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
