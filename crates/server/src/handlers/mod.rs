pub mod users;
pub mod orgs;
pub mod agents;
pub mod projects;
pub mod feed;
pub mod social;
pub mod usage;
pub mod integrations;
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

/// Resolves the authenticated user's internal record, creating user + profile + default org on first login.
/// Always calls upsert to guarantee user and profile exist (idempotent).
/// Ensures a default org exists for every user (personal home org).
pub async fn resolve_user(pool: &sqlx::PgPool, auth: &AuthUser) -> Result<models::User, AppError> {
    let input = models::CreateUserFromToken {
        zero_user_id: auth.user_id.clone(),
        display_name: auth.user_id.clone(),
        profile_image: None,
        primary_zid: None,
    };
    let user = repo::upsert_from_token(pool, &input).await?;

    // Ensure default org exists
    let orgs = aura_network_orgs::repo::list_for_user(pool, user.id).await?;
    if orgs.is_empty() {
        let org_input = aura_network_orgs::models::CreateOrgRequest {
            name: "My Team".to_string(),
            description: None,
            avatar_url: None,
        };
        aura_network_orgs::repo::create(pool, user.id, &user.display_name, &org_input).await?;
    }

    Ok(user)
}
