use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{Agent, CreateAgentRequest, UpdateAgentRequest};

const MAX_LIST_LIMIT: i64 = 100;
const DEFAULT_LIST_LIMIT: i64 = 50;

/// Filters accepted by [`list`]. `listing_status = Some("hireable")`
/// flips the query into the cross-user marketplace view; otherwise
/// the legacy caller-scoped (or org-scoped) behaviour is preserved.
#[derive(Debug, Default, Clone)]
pub struct ListFilters {
    pub org_id: Option<Uuid>,
    pub listing_status: Option<String>,
    pub expertise: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn validate_listing_status(value: &str) -> Result<(), AppError> {
    if matches!(value, "closed" | "hireable") {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Invalid listing_status '{value}'. Must be 'closed' or 'hireable'"
        )))
    }
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    input: &CreateAgentRequest,
) -> Result<Agent, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Agent name must not be empty".into()));
    }

    let skills_json = serde_json::to_value(input.skills.as_deref().unwrap_or(&[]))
        .unwrap_or(serde_json::json!([]));

    let mut tx = pool.begin().await?;

    let machine_type = input.machine_type.as_deref().unwrap_or("local");
    if machine_type != "local" && machine_type != "remote" {
        return Err(AppError::BadRequest(format!(
            "Invalid machine_type: '{machine_type}'. Must be local or remote"
        )));
    }

    let listing_status = input.listing_status.as_deref().unwrap_or("closed");
    validate_listing_status(listing_status)?;

    let expertise = input.expertise.clone().unwrap_or_default();
    let tags = input.tags.clone().unwrap_or_default();

    let agent = sqlx::query_as::<_, Agent>(
        r#"
        INSERT INTO agents (
            user_id, org_id, name, role, personality, system_prompt, skills, icon,
            machine_type, listing_status, expertise, tags
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
    .bind(machine_type)
    .bind(listing_status)
    .bind(&expertise)
    .bind(&tags)
    .fetch_one(&mut *tx)
    .await?;

    // Auto-create agent profile
    sqlx::query(
        r#"
        INSERT INTO profiles (profile_type, agent_id, display_name, avatar)
        VALUES ('agent', $1, $2, $3)
        ON CONFLICT (agent_id) WHERE profile_type = 'agent' AND agent_id IS NOT NULL DO UPDATE SET
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
    filters: &ListFilters,
) -> Result<Vec<Agent>, AppError> {
    // Scoping:
    //   listing_status = Some("hireable")  -> cross-user marketplace view
    //   org_id = Some(_)                   -> org fleet (membership gated upstream)
    //   else                               -> caller's own agents
    //
    // The marketplace view *must* drop the caller-scoped user_id filter,
    // otherwise the page can never surface other users' listings.
    let is_marketplace = filters.listing_status.as_deref() == Some("hireable");

    if let Some(value) = filters.listing_status.as_deref() {
        if !value.is_empty() {
            validate_listing_status(value)?;
        }
    }

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM agents WHERE ");

    if is_marketplace {
        qb.push("listing_status = 'hireable'");
    } else if let Some(org_id) = filters.org_id {
        qb.push("org_id = ");
        qb.push_bind(org_id);
    } else {
        qb.push("user_id = ");
        qb.push_bind(user_id);
    }

    if let Some(slug) = filters.expertise.as_ref().filter(|s| !s.is_empty()) {
        qb.push(" AND expertise @> ARRAY[");
        qb.push_bind(slug.clone());
        qb.push("]::text[]");
    }

    let order_by = match filters.sort.as_deref() {
        Some("latest") => "created_at DESC",
        Some("revenue") => "revenue_usd DESC",
        Some("reputation") => "reputation DESC",
        // "trending" is the default for the marketplace view; the legacy
        // caller-scoped list keeps its historical ascending `created_at`.
        Some("trending") | Some("") | None => {
            if is_marketplace {
                "jobs DESC"
            } else {
                "created_at"
            }
        }
        Some(other) => {
            return Err(AppError::BadRequest(format!(
                "Invalid sort '{other}'. Must be one of trending|latest|revenue|reputation"
            )))
        }
    };
    qb.push(" ORDER BY ");
    qb.push(order_by);

    let limit = filters
        .limit
        .unwrap_or(DEFAULT_LIST_LIMIT)
        .clamp(1, MAX_LIST_LIMIT);
    qb.push(" LIMIT ");
    qb.push_bind(limit);

    if let Some(offset) = filters.offset.filter(|o| *o > 0) {
        qb.push(" OFFSET ");
        qb.push_bind(offset);
    }

    let agents = qb.build_query_as::<Agent>().fetch_all(pool).await?;

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

    if let Some(ref mt) = input.machine_type {
        if mt != "local" && mt != "remote" {
            return Err(AppError::BadRequest(format!(
                "Invalid machine_type: '{mt}'. Must be local or remote"
            )));
        }
    }

    if let Some(ref ls) = input.listing_status {
        validate_listing_status(ls)?;
    }

    let agent = sqlx::query_as::<_, Agent>(
        r#"
        UPDATE agents SET
            name = COALESCE($2, name),
            role = COALESCE($3, role),
            personality = COALESCE($4, personality),
            system_prompt = COALESCE($5, system_prompt),
            skills = COALESCE($6, skills),
            icon = COALESCE($7, icon),
            machine_type = COALESCE($8, machine_type),
            wallet_address = COALESCE($9, wallet_address),
            vm_id = COALESCE($10, vm_id),
            listing_status = COALESCE($11, listing_status),
            expertise = COALESCE($12, expertise),
            tags = COALESCE($13, tags),
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
    .bind(&input.machine_type)
    .bind(&input.wallet_address)
    .bind(&input.vm_id)
    .bind(&input.listing_status)
    .bind(&input.expertise)
    .bind(&input.tags)
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
