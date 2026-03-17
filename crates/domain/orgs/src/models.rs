use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Org {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_user_id: Uuid,
    pub billing_email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrgMember {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub role: String,
    pub credit_budget: Option<i64>,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct OrgInvite {
    pub id: Uuid,
    pub org_id: Uuid,
    pub token: String,
    pub created_by: Uuid,
    pub status: String,
    pub accepted_by: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrgRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrgRequest {
    pub name: Option<String>,
    pub billing_email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberRequest {
    pub role: Option<String>,
    pub credit_budget: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptInviteRequest {
    pub display_name: String,
}
