use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{
    AcceptInviteRequest, CreateOrgRequest, Org, OrgInvite, OrgMember, UpdateMemberRequest,
    UpdateOrgRequest,
};

fn generate_slug(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_hyphen = true; // prevent leading hyphen
    for c in name.to_lowercase().chars() {
        if c.is_alphanumeric() {
            slug.push(c);
            prev_hyphen = false;
        } else if !prev_hyphen {
            slug.push('-');
            prev_hyphen = true;
        }
    }
    let trimmed = slug.trim_matches('-');
    let short_id = &Uuid::new_v4().to_string()[..8];
    format!("{trimmed}-{short_id}")
}

fn validate_role(role: &str) -> Result<(), AppError> {
    match role {
        "owner" | "admin" | "member" => Ok(()),
        _ => Err(AppError::BadRequest(format!("Invalid role: '{role}'. Must be owner, admin, or member"))),
    }
}

pub async fn create(
    pool: &PgPool,
    user_id: Uuid,
    display_name: &str,
    input: &CreateOrgRequest,
) -> Result<Org, AppError> {
    if input.name.trim().is_empty() {
        return Err(AppError::BadRequest("Organization name must not be empty".into()));
    }

    let slug = generate_slug(&input.name);

    let mut tx = pool.begin().await?;

    let org = sqlx::query_as::<_, Org>(
        r#"
        INSERT INTO organizations (name, slug, owner_user_id)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(input.name.trim())
    .bind(&slug)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO org_members (org_id, user_id, display_name, role)
        VALUES ($1, $2, $3, 'owner')
        "#,
    )
    .bind(org.id)
    .bind(user_id)
    .bind(display_name)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(org)
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Org>, AppError> {
    let orgs = sqlx::query_as::<_, Org>(
        r#"
        SELECT o.* FROM organizations o
        JOIN org_members m ON o.id = m.org_id
        WHERE m.user_id = $1
        ORDER BY o.created_at
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(orgs)
}

pub async fn get(pool: &PgPool, org_id: Uuid) -> Result<Org, AppError> {
    sqlx::query_as::<_, Org>("SELECT * FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Organization not found".into()))
}

pub async fn delete(pool: &PgPool, org_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // Delete all org-related data in dependency order (FK constraints)

    // Activity events and comments referencing org
    sqlx::query("DELETE FROM comments WHERE activity_event_id IN (SELECT id FROM activity_events WHERE org_id = $1)")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM activity_events WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    // Token usage
    sqlx::query("DELETE FROM token_usage_daily WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    // Projects
    sqlx::query("DELETE FROM projects WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    // Agent profiles, then agents
    sqlx::query("DELETE FROM follows WHERE follower_profile_id IN (SELECT p.id FROM profiles p JOIN agents a ON p.agent_id = a.id WHERE a.org_id = $1) OR target_profile_id IN (SELECT p.id FROM profiles p JOIN agents a ON p.agent_id = a.id WHERE a.org_id = $1)")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM profiles WHERE agent_id IN (SELECT id FROM agents WHERE org_id = $1)")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM agents WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    // Invites and members
    sqlx::query("DELETE FROM org_invites WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM org_members WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    // Finally the org itself
    let result = sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Organization not found".into()));
    }

    tx.commit().await?;

    Ok(())
}

pub async fn update(
    pool: &PgPool,
    org_id: Uuid,
    input: &UpdateOrgRequest,
) -> Result<Org, AppError> {
    sqlx::query_as::<_, Org>(
        r#"
        UPDATE organizations SET
            name = COALESCE($2, name),
            billing_email = COALESCE($3, billing_email),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(org_id)
    .bind(&input.name)
    .bind(&input.billing_email)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Organization not found".into()))
}

pub async fn list_members(pool: &PgPool, org_id: Uuid) -> Result<Vec<OrgMember>, AppError> {
    let members = sqlx::query_as::<_, OrgMember>(
        "SELECT * FROM org_members WHERE org_id = $1 ORDER BY joined_at",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await?;

    Ok(members)
}

pub async fn get_member(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<OrgMember, AppError> {
    sqlx::query_as::<_, OrgMember>(
        "SELECT * FROM org_members WHERE org_id = $1 AND user_id = $2",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Not a member of this organization".into()))
}

pub async fn update_member(
    pool: &PgPool,
    org_id: Uuid,
    target_user_id: Uuid,
    input: &UpdateMemberRequest,
) -> Result<OrgMember, AppError> {
    if let Some(ref role) = input.role {
        validate_role(role)?;
    }

    sqlx::query_as::<_, OrgMember>(
        r#"
        UPDATE org_members SET
            role = COALESCE($3, role),
            credit_budget = COALESCE($4, credit_budget)
        WHERE org_id = $1 AND user_id = $2
        RETURNING *
        "#,
    )
    .bind(org_id)
    .bind(target_user_id)
    .bind(&input.role)
    .bind(input.credit_budget)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Member not found".into()))
}

pub async fn remove_member(
    pool: &PgPool,
    org_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    // Check they're not the owner
    let member = get_member(pool, org_id, target_user_id).await?;
    if member.role == "owner" {
        return Err(AppError::Forbidden("Cannot remove the owner".into()));
    }

    sqlx::query("DELETE FROM org_members WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(target_user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn create_invite(
    pool: &PgPool,
    org_id: Uuid,
    created_by: Uuid,
) -> Result<OrgInvite, AppError> {
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::days(7);

    let invite = sqlx::query_as::<_, OrgInvite>(
        r#"
        INSERT INTO org_invites (org_id, token, created_by, status, expires_at)
        VALUES ($1, $2, $3, 'pending', $4)
        RETURNING *
        "#,
    )
    .bind(org_id)
    .bind(&token)
    .bind(created_by)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(invite)
}

pub async fn list_invites(pool: &PgPool, org_id: Uuid) -> Result<Vec<OrgInvite>, AppError> {
    let invites = sqlx::query_as::<_, OrgInvite>(
        "SELECT * FROM org_invites WHERE org_id = $1 ORDER BY created_at DESC",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await?;

    Ok(invites)
}

pub async fn revoke_invite(
    pool: &PgPool,
    org_id: Uuid,
    invite_id: Uuid,
) -> Result<OrgInvite, AppError> {
    sqlx::query_as::<_, OrgInvite>(
        r#"
        UPDATE org_invites SET status = 'revoked'
        WHERE id = $1 AND org_id = $2 AND status = 'pending'
        RETURNING *
        "#,
    )
    .bind(invite_id)
    .bind(org_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Invite not found or already used".into()))
}

pub async fn accept_invite(
    pool: &PgPool,
    token: &str,
    user_id: Uuid,
    input: &AcceptInviteRequest,
) -> Result<OrgMember, AppError> {
    let invite = sqlx::query_as::<_, OrgInvite>(
        "SELECT * FROM org_invites WHERE token = $1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Invite not found".into()))?;

    if invite.status != "pending" {
        return Err(AppError::BadRequest("Invite is no longer valid".into()));
    }

    if Utc::now() > invite.expires_at {
        sqlx::query("UPDATE org_invites SET status = 'expired' WHERE id = $1")
            .bind(invite.id)
            .execute(pool)
            .await?;
        return Err(AppError::BadRequest("Invite has expired".into()));
    }

    // Check not already a member
    let existing = sqlx::query_as::<_, OrgMember>(
        "SELECT * FROM org_members WHERE org_id = $1 AND user_id = $2",
    )
    .bind(invite.org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Already a member of this organization".into()));
    }

    // Accept invite and create membership atomically
    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE org_invites SET status = 'accepted', accepted_by = $2, accepted_at = NOW() WHERE id = $1",
    )
    .bind(invite.id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    let member = sqlx::query_as::<_, OrgMember>(
        r#"
        INSERT INTO org_members (org_id, user_id, display_name, role)
        VALUES ($1, $2, $3, 'member')
        RETURNING *
        "#,
    )
    .bind(invite.org_id)
    .bind(user_id)
    .bind(&input.display_name)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(member)
}

/// Check that a user has at least the given minimum role in an org.
/// Role hierarchy: owner > admin > member
pub async fn require_role(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    min_role: &str,
) -> Result<OrgMember, AppError> {
    let member = get_member(pool, org_id, user_id).await?;

    let role_level = |r: &str| -> u8 {
        match r {
            "owner" => 3,
            "admin" => 2,
            "member" => 1,
            _ => 0,
        }
    };

    if role_level(&member.role) < role_level(min_role) {
        return Err(AppError::Forbidden(format!(
            "Requires at least '{}' role",
            min_role
        )));
    }

    Ok(member)
}
