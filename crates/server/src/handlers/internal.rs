// Integration contract with aura-swarm:
// - User lookup uses zero_user_id (text string from zOS), not internal UUID
// - Agent IDs in usage/activity payloads must be aura-network UUIDs
//   (swarm should look up agent UUID via API, not use its internal blake3 hash)
// - All payloads use camelCase JSON

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;

use aura_network_auth::InternalAuth;
use aura_network_core::AppError;

use crate::state::AppState;

pub async fn get_user_by_zero_id(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Path(zero_user_id): Path<String>,
) -> Result<Json<aura_network_users::models::User>, AppError> {
    let user =
        aura_network_users::handlers::get_user_by_zero_id(&state.pool, &zero_user_id).await?;
    Ok(Json(user))
}

pub async fn post_activity(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Json(mut input): Json<aura_network_feed::models::CreateActivityEventRequest>,
) -> Result<Json<aura_network_feed::models::ActivityEvent>, AppError> {
    // Resolve profile_id from user_id. The caller (orbit) sends the zOS user UUID
    // from the JWT, which is stored as zero_user_id in the users table.
    // Look up the user by zero_user_id, then get their profile.
    if let Some(user_id) = input.user_id {
        if let Ok(user) =
            aura_network_users::repo::get_by_zero_id(&state.pool, &user_id.to_string()).await
        {
            if let Ok(profile) =
                aura_network_users::repo::get_profile_by_user_id(&state.pool, user.id).await
            {
                input.profile_id = profile.id;
            }
        }
    }
    let event = aura_network_feed::handlers::post_activity(&state.pool, input).await?;

    // Broadcast to WebSocket clients
    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "type": "activity.new",
        "data": &event
    })) {
        let _ = state.events_tx.send(json);
    }

    Ok(Json(event))
}

pub async fn record_usage(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Json(mut input): Json<aura_network_usage::models::RecordUsageRequest>,
) -> Result<StatusCode, AppError> {
    // If zero_user_id is provided, resolve to internal user ID.
    // aura-router sends the zOS UUID, but token_usage_daily.user_id
    // references users.id (aura-network's internal UUID).
    if let Some(ref zero_id) = input.zero_user_id {
        if let Ok(user) =
            aura_network_users::handlers::get_user_by_zero_id(&state.pool, zero_id).await
        {
            input.user_id = user.id;
        }
    }

    aura_network_usage::handlers::record_usage(&state.pool, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn check_budget(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<aura_network_usage::models::BudgetStatus>, AppError> {
    let status = aura_network_usage::handlers::check_budget(&state.pool, org_id, user_id).await?;
    Ok(Json(status))
}

pub async fn get_project_usage(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    axum::extract::Query(query): axum::extract::Query<aura_network_usage::models::UsageQuery>,
) -> Result<Json<aura_network_usage::models::UsageSummary>, AppError> {
    let usage = aura_network_usage::handlers::get_project_usage(
        &state.pool,
        project_id,
        query.period.as_deref(),
    )
    .await?;
    Ok(Json(usage))
}

pub async fn get_org_usage(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    axum::extract::Query(query): axum::extract::Query<aura_network_usage::models::UsageQuery>,
) -> Result<Json<aura_network_usage::models::UsageSummary>, AppError> {
    let usage = aura_network_usage::handlers::get_org_usage(
        &state.pool,
        org_id,
        query.period.as_deref(),
    )
    .await?;
    Ok(Json(usage))
}

pub async fn get_network_usage(
    _auth: InternalAuth,
    State(state): State<AppState>,
) -> Result<Json<aura_network_usage::models::UsageSummary>, AppError> {
    // Network-wide: sum all usage
    let usage = sqlx::query_as::<_, aura_network_usage::models::UsageSummary>(
        r#"
        SELECT
            COALESCE(SUM(input_tokens), 0)::int8 as total_input_tokens,
            COALESCE(SUM(output_tokens), 0)::int8 as total_output_tokens,
            COALESCE(SUM(input_tokens + output_tokens), 0)::int8 as total_tokens,
            COALESCE(SUM(estimated_cost_usd)::float8, 0.0) as total_cost_usd
        FROM token_usage_daily
        "#,
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(usage))
}

pub async fn list_org_integrations(
    _auth: InternalAuth,
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<aura_network_integrations::models::OrgIntegration>>, AppError> {
    let integrations = aura_network_integrations::repo::list(&state.pool, org_id).await?;
    Ok(Json(integrations))
}
