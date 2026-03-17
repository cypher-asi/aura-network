use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{Agent, CreateAgentRequest, UpdateAgentRequest};

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    input: &CreateAgentRequest,
) -> Result<Agent, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Agent name must not be empty".into()));
    }

    let skills_json = serde_json::to_value(
        input.skills.as_deref().unwrap_or(&[]),
    )
    .unwrap_or(serde_json::json!([]));

    let mut tx = pool.begin().await?;

    let agent = sqlx::query_as::<_, Agent>(
        r#"
        INSERT INTO agents (user_id, org_id, name, role, personality, system_prompt, skills, icon)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(input.org_id)
    .bind(input.name.trim())
    .bind(&input.role)
    .bind(&input.personality)
    .bind(&input.system_prompt)
    .bind(&skills_json)
    .bind(&input.icon)
    .fetch_one(&mut *tx)
    .await?;

    // Auto-create agent profile
    sqlx::query(
        r#"
        INSERT INTO profiles (profile_type, agent_id, display_name, avatar)
        VALUES ('agent', $1, $2, $3)
        ON CONFLICT (agent_id) WHERE profile_type = 'agent' DO UPDATE SET
            display_name = EXCLUDED.display_name,
            avatar = EXCLUDED.avatar,
            updated_at = NOW()
        "#,
    )
    .bind(agent.id)
    .bind(&agent.name)
    .bind(&agent.icon)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(agent)
}

pub async fn list(
    pool: &PgPool,
    user_id: Uuid,
    org_id: Option<Uuid>,
) -> Result<Vec<Agent>, AppError> {
    let agents = if let Some(org_id) = org_id {
        sqlx::query_as::<_, Agent>(
            "SELECT * FROM agents WHERE user_id = $1 AND org_id = $2 ORDER BY created_at",
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Agent>(
            "SELECT * FROM agents WHERE user_id = $1 ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?
    };

    Ok(agents)
}

pub async fn get(pool: &PgPool, agent_id: Uuid) -> Result<Agent, AppError> {
    sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Agent not found".into()))
}

pub async fn update(
    pool: &PgPool,
    agent_id: Uuid,
    user_id: Uuid,
    input: &UpdateAgentRequest,
) -> Result<Agent, AppError> {
    let existing = get(pool, agent_id).await?;
    if existing.user_id != user_id {
        return Err(AppError::Forbidden("Not the owner of this agent".into()));
    }

    let skills_json = input
        .skills
        .as_ref()
        .map(|s| serde_json::to_value(s).unwrap_or(serde_json::json!([])));

    let agent = sqlx::query_as::<_, Agent>(
        r#"
        UPDATE agents SET
            name = COALESCE($2, name),
            role = COALESCE($3, role),
            personality = COALESCE($4, personality),
            system_prompt = COALESCE($5, system_prompt),
            skills = COALESCE($6, skills),
            icon = COALESCE($7, icon),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(agent_id)
    .bind(&input.name)
    .bind(&input.role)
    .bind(&input.personality)
    .bind(&input.system_prompt)
    .bind(&skills_json)
    .bind(&input.icon)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Agent not found".into()))?;

    // Update agent profile
    sqlx::query(
        r#"
        UPDATE profiles SET
            display_name = $2,
            avatar = $3,
            updated_at = NOW()
        WHERE profile_type = 'agent' AND agent_id = $1
        "#,
    )
    .bind(agent_id)
    .bind(&agent.name)
    .bind(&agent.icon)
    .execute(pool)
    .await?;

    Ok(agent)
}

pub async fn delete(pool: &PgPool, agent_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let existing = get(pool, agent_id).await?;
    if existing.user_id != user_id {
        return Err(AppError::Forbidden("Not the owner of this agent".into()));
    }

    let mut tx = pool.begin().await?;

    // Delete profile first (FK constraint)
    sqlx::query("DELETE FROM profiles WHERE profile_type = 'agent' AND agent_id = $1")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM agents WHERE id = $1")
        .bind(agent_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
