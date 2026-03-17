use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_projects::{handlers, models};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ProjectListQuery {
    pub org_id: Uuid,
}

pub async fn create_project(
    _auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::CreateProjectRequest>,
) -> Result<Json<models::Project>, AppError> {
    let project = handlers::create_project(&state.pool, input).await?;
    Ok(Json(project))
}

pub async fn list_projects(
    _auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<ProjectListQuery>,
) -> Result<Json<Vec<models::Project>>, AppError> {
    let projects = handlers::list_projects(&state.pool, query.org_id).await?;
    Ok(Json(projects))
}

pub async fn get_project(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<models::Project>, AppError> {
    let project = handlers::get_project(&state.pool, project_id).await?;
    Ok(Json(project))
}

pub async fn update_project(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(input): Json<models::UpdateProjectRequest>,
) -> Result<Json<models::Project>, AppError> {
    let project = handlers::update_project(&state.pool, project_id, input).await?;
    Ok(Json(project))
}

pub async fn delete_project(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<(), AppError> {
    handlers::delete_project(&state.pool, project_id).await
}
