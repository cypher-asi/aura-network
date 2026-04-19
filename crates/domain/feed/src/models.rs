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
    pub post_type: String,
    pub title: String,
    pub summary: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub agent_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub push_id: Option<Uuid>,
    pub commit_ids: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    #[sqlx(default)]
    pub comment_count: i64,
    // Vote aggregates. Populated for feedback items via join; always zero for
    // non-feedback event types so older clients that don't care can ignore
    // them.
    #[serde(default)]
    #[sqlx(default)]
    pub upvotes: i64,
    #[serde(default)]
    #[sqlx(default)]
    pub downvotes: i64,
    #[serde(default)]
    #[sqlx(default)]
    pub vote_score: i64,
    /// Current viewer's vote on this item: "up", "down", or "none".
    #[sqlx(default)]
    pub viewer_vote: String,
}

/// Summary of vote aggregates for a single feedback post from the current
/// viewer's perspective. Returned by vote mutations and the summary endpoint.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSummary {
    pub upvotes: i64,
    pub downvotes: i64,
    pub score: i64,
    pub viewer_vote: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CastVoteRequest {
    /// "up", "down", or "none". `none` clears any existing vote for the
    /// current viewer.
    pub vote: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPostRequest {
    /// Shallow-merged into the existing `metadata` JSON object. Keys with a
    /// JSON `null` value are removed. Provided to support feedback status
    /// updates without introducing event-specific mutation endpoints.
    pub metadata: Option<serde_json::Value>,
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
    /// Optional: when the client omits this, the HTTP handler resolves the
    /// viewer's profile from their JWT. Lets thin proxies like aura-os-server
    /// post on behalf of a user without threading the profile id through
    /// their own session state.
    #[serde(default)]
    pub profile_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub event_type: String,
    pub post_type: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub agent_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub push_id: Option<Uuid>,
    pub commit_ids: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCommentRequest {
    pub content: String,
}

/// Entry returned by `GET /api/public/feedback`. Shape is stable and
/// intentionally narrower than `ActivityEvent` — marketing / roadmap
/// surfaces read this without auth, so it's derived from already-public
/// fields only and bakes the feedback-specific metadata out of
/// `activity_events.metadata` into flat keys.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PublicFeedbackEntry {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub category: String,
    pub status: String,
    pub upvotes: i64,
    pub downvotes: i64,
    pub vote_score: i64,
    pub comment_count: i64,
    pub created_at: DateTime<Utc>,
    pub author_name: Option<String>,
    pub author_avatar: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicFeedbackQuery {
    pub sort: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    /// Optional product filter. Defaults to `"aura"` — the OS feedback surface.
    /// Left open so the same endpoint can serve future products without a
    /// schema change.
    pub product: Option<String>,
}

impl PublicFeedbackQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(100).min(200).max(1)
    }

    pub fn product(&self) -> &str {
        self.product.as_deref().unwrap_or("aura")
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedQuery {
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl FeedQuery {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).min(100).max(1)
    }

    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

const VALID_EVENT_TYPES: &[&str] = &[
    "commit",
    "task_completed",
    "task_failed",
    "loop_started",
    "loop_finished",
    "agent_created",
    "post",
    "push",
    "feedback",
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
