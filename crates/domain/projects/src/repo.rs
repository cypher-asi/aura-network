use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{CreateProjectRequest, Project, UpdateProjectRequest};

pub async fn create(pool: &PgPool, input: &CreateProjectRequest) -> Result<Project, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Project name must not be empty".into(),
        ));
    }

    let visibility = input.visibility.as_deref().unwrap_or("private");
    if visibility != "public" && visibility != "private" {
        return Err(AppError::BadRequest(format!(
            "Invalid visibility: '{visibility}'. Must be public or private"
        )));
    }

    let project = sqlx::query_as::<_, Project>(
        r#"
        INSERT INTO projects (org_id, name, description, folder, visibility)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(input.org_id)
    .bind(input.name.trim())
    .bind(&input.description)
    .bind(&input.folder)
    .bind(visibility)
    .fetch_one(pool)
    .await?;

    Ok(project)
}

pub async fn list(pool: &PgPool, org_id: Uuid) -> Result<Vec<Project>, AppError> {
    let projects = sqlx::query_as::<_, Project>(
        "SELECT * FROM projects WHERE org_id = $1 ORDER BY created_at",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

pub async fn get(pool: &PgPool, project_id: Uuid) -> Result<Project, AppError> {
    sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".into()))
}

pub async fn update(
    pool: &PgPool,
    project_id: Uuid,
    input: &UpdateProjectRequest,
) -> Result<Project, AppError> {
    if let Some(ref status) = input.status {
        match status.as_str() {
            "active" | "archived" => {}
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Invalid project status: '{status}'. Must be active or archived"
                )))
            }
        }
    }

    if let Some(ref visibility) = input.visibility {
        if visibility != "public" && visibility != "private" {
            return Err(AppError::BadRequest(format!(
                "Invalid visibility: '{visibility}'. Must be public or private"
            )));
        }
    }

    sqlx::query_as::<_, Project>(
        r#"
        UPDATE projects SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            folder = COALESCE($4, folder),
            status = COALESCE($5, status),
            visibility = COALESCE($6, visibility),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.folder)
    .bind(&input.status)
    .bind(&input.visibility)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Project not found".into()))
}

pub async fn delete(pool: &PgPool, project_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Project not found".into()));
    }

    Ok(())
}
