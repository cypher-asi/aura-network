use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEvent {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub org_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub event_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: Uuid,
    pub activity_event_id: Uuid,
    pub profile_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivityEventRequest {
    pub profile_id: Uuid,
    pub org_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub event_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedQuery {
    pub filter: Option<String>,
    #[serde(flatten)]
    pub pagination: aura_network_core::PaginationParams,
}

const VALID_EVENT_TYPES: &[&str] = &[
    "commit",
    "task_completed",
    "task_failed",
    "loop_started",
    "loop_finished",
    "agent_created",
];

pub fn validate_event_type(event_type: &str) -> Result<(), aura_network_core::AppError> {
    if VALID_EVENT_TYPES.contains(&event_type) {
        Ok(())
    } else {
        Err(aura_network_core::AppError::BadRequest(format!(
            "Invalid event type: '{event_type}'"
        )))
    }
}
