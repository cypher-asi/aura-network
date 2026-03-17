use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{Agent, CreateAgentRequest, UpdateAgentRequest};
use crate::repo;

pub async fn create_agent(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateAgentRequest,
) -> Result<Agent, AppError> {
    repo::create(pool, user_id, &input).await
}

pub async fn list_agents(
    pool: &PgPool,
    user_id: Uuid,
    org_id: Option<Uuid>,
) -> Result<Vec<Agent>, AppError> {
    repo::list(pool, user_id, org_id).await
}

pub async fn get_agent(pool: &PgPool, agent_id: Uuid) -> Result<Agent, AppError> {
    repo::get(pool, agent_id).await
}

pub async fn update_agent(
    pool: &PgPool,
    agent_id: Uuid,
    user_id: Uuid,
    input: UpdateAgentRequest,
) -> Result<Agent, AppError> {
    repo::update(pool, agent_id, user_id, &input).await
}

pub async fn delete_agent(
    pool: &PgPool,
    agent_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    repo::delete(pool, agent_id, user_id).await
}
