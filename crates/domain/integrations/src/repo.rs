use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{CreateIntegrationRequest, OrgIntegration, UpdateIntegrationRequest};

pub async fn create(
    pool: &PgPool,
    org_id: Uuid,
    input: &CreateIntegrationRequest,
) -> Result<OrgIntegration, AppError> {
    if input.integration_type.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Integration type must not be empty".into(),
        ));
    }

    let enabled = input.enabled.unwrap_or(true);

    let integration = sqlx::query_as::<_, OrgIntegration>(
        r#"
        INSERT INTO org_integrations (org_id, integration_type, config, enabled)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(org_id)
    .bind(input.integration_type.trim())
    .bind(&input.config)
    .bind(enabled)
    .fetch_one(pool)
    .await?;

    Ok(integration)
}

pub async fn list(pool: &PgPool, org_id: Uuid) -> Result<Vec<OrgIntegration>, AppError> {
    let integrations = sqlx::query_as::<_, OrgIntegration>(
        "SELECT * FROM org_integrations WHERE org_id = $1 ORDER BY created_at",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await?;

    Ok(integrations)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<OrgIntegration, AppError> {
    sqlx::query_as::<_, OrgIntegration>("SELECT * FROM org_integrations WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Integration not found".into()))
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    input: &UpdateIntegrationRequest,
) -> Result<OrgIntegration, AppError> {
    sqlx::query_as::<_, OrgIntegration>(
        r#"
        UPDATE org_integrations SET
            config = COALESCE($2, config),
            enabled = COALESCE($3, enabled),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(&input.config)
    .bind(input.enabled)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Integration not found".into()))
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM org_integrations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Integration not found".into()));
    }

    Ok(())
}
