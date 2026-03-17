use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{CreateProjectRequest, Project, UpdateProjectRequest};
use crate::repo;

pub async fn create_project(
    pool: &PgPool,
    input: CreateProjectRequest,
) -> Result<Project, AppError> {
    repo::create(pool, &input).await
}

pub async fn list_projects(pool: &PgPool, org_id: Uuid) -> Result<Vec<Project>, AppError> {
    repo::list(pool, org_id).await
}

pub async fn get_project(pool: &PgPool, project_id: Uuid) -> Result<Project, AppError> {
    repo::get(pool, project_id).await
}

pub async fn update_project(
    pool: &PgPool,
    project_id: Uuid,
    input: UpdateProjectRequest,
) -> Result<Project, AppError> {
    repo::update(pool, project_id, &input).await
}

pub async fn delete_project(pool: &PgPool, project_id: Uuid) -> Result<(), AppError> {
    repo::delete(pool, project_id).await
}
