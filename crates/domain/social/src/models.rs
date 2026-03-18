use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub follower_profile_id: Uuid,
    pub target_profile_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowRequest {
    pub target_profile_id: Uuid,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub profile_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub profile_type: String,
    pub tokens_used: i64,
    pub estimated_cost_usd: f64,
    pub event_count: i64,
}
