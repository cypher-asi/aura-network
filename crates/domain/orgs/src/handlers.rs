use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::*;
use crate::repo;

pub async fn create_org(
    pool: &PgPool,
    user_id: Uuid,
    display_name: &str,
    input: CreateOrgRequest,
) -> Result<Org, AppError> {
    repo::create(pool, user_id, display_name, &input).await
}

pub async fn list_orgs(pool: &PgPool, user_id: Uuid) -> Result<Vec<Org>, AppError> {
    repo::list_for_user(pool, user_id).await
}

pub async fn get_org(pool: &PgPool, org_id: Uuid) -> Result<Org, AppError> {
    repo::get(pool, org_id).await
}

pub async fn delete_org(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Uuid,
) -> Result<(), AppError> {
    repo::require_role(pool, org_id, actor_user_id, "owner").await?;
    repo::delete(pool, org_id).await
}

pub async fn update_org(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Uuid,
    input: UpdateOrgRequest,
) -> Result<Org, AppError> {
    repo::require_role(pool, org_id, actor_user_id, "admin").await?;
    repo::update(pool, org_id, &input).await
}

pub async fn list_members(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Uuid,
) -> Result<Vec<OrgMember>, AppError> {
    repo::get_member(pool, org_id, actor_user_id).await?; // verify membership
    repo::list_members(pool, org_id).await
}

pub async fn update_member(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Uuid,
    target_user_id: Uuid,
    input: UpdateMemberRequest,
) -> Result<OrgMember, AppError> {
    repo::require_role(pool, org_id, actor_user_id, "admin").await?;
    repo::update_member(pool, org_id, target_user_id, &input).await
}

pub async fn remove_member(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    repo::require_role(pool, org_id, actor_user_id, "admin").await?;
    repo::remove_member(pool, org_id, target_user_id).await
}

pub async fn create_invite(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Uuid,
) -> Result<OrgInvite, AppError> {
    repo::require_role(pool, org_id, actor_user_id, "admin").await?;
    repo::create_invite(pool, org_id, actor_user_id).await
}

pub async fn list_invites(
    pool: &PgPool,
    org_id: Uuid,
    actor_user_id: Uuid,
) -> Result<Vec<OrgInvite>, AppError> {
    repo::require_role(pool, org_id, actor_user_id, "admin").await?;
    repo::list_invites(pool, org_id).await
}

pub async fn revoke_invite(
    pool: &PgPool,
    org_id: Uuid,
    invite_id: Uuid,
    actor_user_id: Uuid,
) -> Result<OrgInvite, AppError> {
    repo::require_role(pool, org_id, actor_user_id, "admin").await?;
    repo::revoke_invite(pool, org_id, invite_id).await
}

pub async fn accept_invite(
    pool: &PgPool,
    token: &str,
    user_id: Uuid,
    input: AcceptInviteRequest,
) -> Result<OrgMember, AppError> {
    repo::accept_invite(pool, token, user_id, &input).await
}
