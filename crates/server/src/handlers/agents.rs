use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_agents::{handlers, models};
use aura_network_users::repo as user_repo;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AgentListQuery {
    pub org_id: Option<Uuid>,
}

pub async fn create_agent(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::CreateAgentRequest>,
) -> Result<Json<models::Agent>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let agent = handlers::create_agent(&state.pool, user.id, input).await?;
    Ok(Json(agent))
}

pub async fn list_agents(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<AgentListQuery>,
) -> Result<Json<Vec<models::Agent>>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let agents = handlers::list_agents(&state.pool, user.id, query.org_id).await?;
    Ok(Json(agents))
}

pub async fn get_agent(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<models::Agent>, AppError> {
    let agent = handlers::get_agent(&state.pool, agent_id).await?;
    Ok(Json(agent))
}

pub async fn update_agent(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(input): Json<models::UpdateAgentRequest>,
) -> Result<Json<models::Agent>, AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    let agent = handlers::update_agent(&state.pool, agent_id, user.id, input).await?;
    Ok(Json(agent))
}

pub async fn delete_agent(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<(), AppError> {
    let user = user_repo::get_by_zero_id(&state.pool, &auth.user_id).await?;
    handlers::delete_agent(&state.pool, agent_id, user.id).await
}
