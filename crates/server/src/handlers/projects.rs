use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_orgs::repo as org_repo;
use aura_network_projects::{handlers, models};

use crate::state::AppState;
use super::resolve_user;

#[derive(Debug, Deserialize)]
pub struct ProjectListQuery {
    pub org_id: Uuid,
}

pub async fn create_project(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::CreateProjectRequest>,
) -> Result<Json<models::Project>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
    org_repo::get_member(&state.pool, input.org_id, user.id).await?;
    let project = handlers::create_project(&state.pool, input).await?;
    Ok(Json(project))
}

pub async fn list_projects(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<Vec<models::Project>>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
    org_repo::get_member(&state.pool, query.org_id, user.id).await?;
    let projects = handlers::list_projects(&state.pool, query.org_id).await?;
    Ok(Json(projects))
}

pub async fn get_project(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<models::Project>, AppError> {
    let user = resolve_user(&state.pool, &auth).await?;
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
    let user = resolve_user(&state.pool, &auth).await?;
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
    let user = resolve_user(&state.pool, &auth).await?;
    let existing = handlers::get_project(&state.pool, project_id).await?;
    org_repo::require_role(&state.pool, existing.org_id, user.id, "admin").await?;
    handlers::delete_project(&state.pool, project_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
