use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub fn default_permissions() -> serde_json::Value {
    json!({
        "scope": {
            "orgs": [],
            "projects": [],
            "agent_ids": []
        },
        "capabilities": []
    })
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    pub name: String,
    pub role: Option<String>,
    pub personality: Option<String>,
    pub system_prompt: Option<String>,
    pub skills: serde_json::Value,
    pub permissions: serde_json::Value,
    pub icon: Option<String>,
    pub machine_type: String,
    pub wallet_address: Option<String>,
    pub vm_id: Option<String>,
    pub last_active_at: Option<DateTime<Utc>>,
    /// Marketplace discoverability. `"closed"` (default) or
    /// `"hireable"`. Hireable rows surface in the cross-user
    /// `GET /api/agents?listing_status=hireable` view.
    pub listing_status: String,
    /// Marketplace expertise slugs (e.g. `"coding"`, `"devops"`).
    pub expertise: Vec<String>,
    /// Aggregated marketplace stats. Server-computed; clients should
    /// treat them as read-only on create/update.
    pub jobs: i64,
    pub revenue_usd: f64,
    pub reputation: f32,
    /// Free-form tags. Used by aura-os-server's legacy dual-write
    /// fallback (`listing_status:hireable`, `expertise:<slug>`) so
    /// older readers can still observe marketplace state.
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentRequest {
    pub org_id: Option<Uuid>,
    pub name: String,
    pub role: Option<String>,
    pub personality: Option<String>,
    pub system_prompt: Option<String>,
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub permissions: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub machine_type: Option<String>,
    /// `"closed"` or `"hireable"`. Defaults to `"closed"` when omitted.
    pub listing_status: Option<String>,
    pub expertise: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub personality: Option<String>,
    pub system_prompt: Option<String>,
    pub skills: Option<Vec<String>>,
    pub permissions: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub machine_type: Option<String>,
    /// Patch-style: `None` leaves the stored value, `Some("hireable")`
    /// flips it. Server validates against `closed`/`hireable`.
    pub listing_status: Option<String>,
    /// `None` leaves the stored array, `Some(vec)` replaces it
    /// wholesale (matching aura-os-server's "replace tags wholesale"
    /// semantics).
    pub expertise: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    /// Server-set only — ignored in user-facing deserialization.
    #[serde(skip_deserializing)]
    pub wallet_address: Option<String>,
    /// Server-set only — ignored in user-facing deserialization.
    #[serde(skip_deserializing)]
    pub vm_id: Option<String>,
}
