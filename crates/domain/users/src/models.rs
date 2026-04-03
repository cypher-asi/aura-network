use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub zero_user_id: String,
    pub display_name: String,
    pub profile_image: Option<String>,
    pub primary_zid: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub is_access_granted: bool,
    pub access_granted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: Uuid,
    pub profile_type: String,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    #[serde(flatten)]
    pub user: User,
    pub profile_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub profile_image: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AppAccessCode {
    pub id: Uuid,
    pub code: String,
    pub created_by: Uuid,
    pub redeemed_by: Option<Uuid>,
    pub status: String,
    pub redeemed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeemAccessCodeRequest {
    pub code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserFromToken {
    pub zero_user_id: String,
    pub display_name: String,
    pub profile_image: Option<String>,
    pub primary_zid: Option<String>,
}
