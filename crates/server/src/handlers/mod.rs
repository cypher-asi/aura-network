pub mod users;
pub mod orgs;
pub mod agents;
pub mod projects;
pub mod feed;
pub mod social;
pub mod usage;
pub mod internal;
pub mod ws;

use axum::Json;

use aura_network_auth::AuthUser;
use aura_network_core::AppError;
use aura_network_users::{models, repo};

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Resolves the authenticated user's internal record, creating user + profile on first login.
/// Always calls upsert to guarantee both user and profile exist (idempotent).
pub async fn resolve_user(pool: &sqlx::PgPool, auth: &AuthUser) -> Result<models::User, AppError> {
    let input = models::CreateUserFromToken {
        zero_user_id: auth.user_id.clone(),
        display_name: auth.user_id.clone(),
        profile_image: None,
        primary_zid: None,
    };
    repo::upsert_from_token(pool, &input).await
}
