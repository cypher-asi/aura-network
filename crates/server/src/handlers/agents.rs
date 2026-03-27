use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use aura_network_agents::{handlers, models};
use aura_network_auth::AuthUser;
use aura_network_core::AppError;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AgentListQuery {
    pub org_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct WalletInitResponse {
    address: String,
}

pub async fn create_agent(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(input): Json<models::CreateAgentRequest>,
) -> Result<Json<models::Agent>, AppError> {
    let user = super::resolve_user(&state.pool, &auth).await?;
    let mut agent = handlers::create_agent(&state.pool, user.id, input).await?;

    // Automatically create an EIP7702 wallet for the agent via zOS API
    if let (Some(ref zos_url), Some(ref zos_token)) =
        (&state.zos_api_url, &state.zos_api_internal_token)
    {
        let agent_id = agent.id.to_string();
        match state
            .http_client
            .post(format!("{zos_url}/thirdweb/initialize-agent-wallet"))
            .header("x-internal-token", zos_token)
            .json(&serde_json::json!({ "agentId": agent_id, "ownerUserId": user.id }))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(wallet) = resp.json::<WalletInitResponse>().await {
                    let update = models::UpdateAgentRequest {
                        wallet_address: Some(wallet.address),
                        ..Default::default()
                    };
                    if let Ok(updated) =
                        handlers::update_agent(&state.pool, agent.id, user.id, update).await
                    {
                        agent = updated;
                    }
                }
            }
            Ok(resp) => {
                tracing::warn!(
                    agent_id = %agent.id,
                    status = %resp.status(),
                    "Failed to create agent wallet via zOS API"
                );
            }
            Err(e) => {
                tracing::warn!(
                    agent_id = %agent.id,
                    error = %e,
                    "Failed to reach zOS API for agent wallet creation"
                );
            }
        }
    }

    Ok(Json(agent))
}

pub async fn list_agents(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(query): Query<AgentListQuery>,
) -> Result<Json<Vec<models::Agent>>, AppError> {
    let user = super::resolve_user(&state.pool, &auth).await?;
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
    let user = super::resolve_user(&state.pool, &auth).await?;
    let agent = handlers::update_agent(&state.pool, agent_id, user.id, input).await?;
    Ok(Json(agent))
}

pub async fn delete_agent(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let user = super::resolve_user(&state.pool, &auth).await?;
    handlers::delete_agent(&state.pool, agent_id, user.id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
