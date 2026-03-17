use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_orgs::{handlers, models};
use aura_network_users::repo as user_repo;

use crate::state::AppState;

pub async fn create_org(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::CreateOrgRequest>,
) -> Result<Json<models::Org>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let org = handlers::create_org(&state.pool, user.id, &user.display_name, input).await?;
    Ok(Json(org))
}

pub async fn list_orgs(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<models::Org>>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let orgs = handlers::list_orgs(&state.pool, user.id).await?;
    Ok(Json(orgs))
}

pub async fn get_org(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<models::Org>, AppError> {
    let org = handlers::get_org(&state.pool, org_id).await?;
    Ok(Json(org))
}

pub async fn update_org(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(input): Json<models::UpdateOrgRequest>,
) -> Result<Json<models::Org>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let org = handlers::update_org(&state.pool, org_id, user.id, input).await?;
    Ok(Json(org))
}

pub async fn list_members(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<models::OrgMember>>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let members = handlers::list_members(&state.pool, org_id, user.id).await?;
    Ok(Json(members))
}

pub async fn update_member(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<models::UpdateMemberRequest>,
) -> Result<Json<models::OrgMember>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let member = handlers::update_member(&state.pool, org_id, user.id, target_user_id, input).await?;
    Ok(Json(member))
}

pub async fn remove_member(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    handlers::remove_member(&state.pool, org_id, user.id, target_user_id).await
}

pub async fn create_invite(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<models::OrgInvite>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let invite = handlers::create_invite(&state.pool, org_id, user.id).await?;
    Ok(Json(invite))
}

pub async fn list_invites(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<models::OrgInvite>>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let invites = handlers::list_invites(&state.pool, org_id, user.id).await?;
    Ok(Json(invites))
}

pub async fn revoke_invite(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((org_id, invite_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<models::OrgInvite>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let invite = handlers::revoke_invite(&state.pool, org_id, invite_id, user.id).await?;
    Ok(Json(invite))
}

pub async fn accept_invite(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(input): Json<models::AcceptInviteRequest>,
) -> Result<Json<models::OrgMember>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let member = handlers::accept_invite(&state.pool, &token, user.id, input).await?;
    Ok(Json(member))
}
