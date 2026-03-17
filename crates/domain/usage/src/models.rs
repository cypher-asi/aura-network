use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageDaily {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub model: String,
    pub date: NaiveDate,
    pub input_tokens: i64,
    pub output_tokens: i64,
    // Note: stored as NUMERIC(10,4) in DB, cast to float8 in aggregate queries
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStats {
    pub id: Uuid,
    pub date: NaiveDate,
    pub daily_active_users: i32,
    pub total_users: i32,
    pub new_signups: i32,
    pub projects_created: i32,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_revenue_usd: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordUsageRequest {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MemberUsage {
    pub user_id: Uuid,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetStatus {
    pub allowed: bool,
    pub budget: Option<i64>,
    pub used: i64,
    pub remaining: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageQuery {
    pub period: Option<String>,
}
