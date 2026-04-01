pub mod agents;
pub mod feed;
pub mod integrations;
pub mod internal;
pub mod orgs;
pub mod projects;
pub mod social;
pub mod usage;
pub mod users;
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
/// If the display name is a UUID placeholder, fetches the real name from zOS API.
pub async fn resolve_user(
    state: &crate::state::AppState,
    auth: &AuthUser,
) -> Result<models::User, AppError> {
    let input = models::CreateUserFromToken {
        zero_user_id: auth.user_id.clone(),
        display_name: auth.user_id.clone(),
        profile_image: None,
        primary_zid: None,
    };
    let mut user = repo::upsert_from_token(&state.pool, &input).await?;

    // Ensure default org exists
    let orgs = aura_network_orgs::repo::list_for_user(&state.pool, user.id).await?;
    if orgs.is_empty() {
        let org_input = aura_network_orgs::models::CreateOrgRequest {
            name: "My Team".to_string(),
            description: None,
            avatar_url: None,
        };
        aura_network_orgs::repo::create(&state.pool, user.id, &user.display_name, &org_input)
            .await?;
    }

    // If display name is still the zOS user ID placeholder, fetch the real name
    if user.display_name == auth.user_id {
        if let Some(name) = fetch_display_name_from_zos(state, &auth.token).await {
            let update = models::UpdateUserRequest {
                display_name: Some(name.clone()),
                bio: None,
                profile_image: None,
                location: None,
                website: None,
            };
            if let Ok(updated) = repo::update(&state.pool, user.id, &update).await {
                user = updated;
                // Also update the profile display_name to match
                let _ = repo::update_profile_display_name(
                    &state.pool,
                    user.id,
                    &name,
                )
                .await;
            }
        }
    }

    Ok(user)
}

async fn fetch_display_name_from_zos(
    state: &crate::state::AppState,
    token: &str,
) -> Option<String> {
    let zos_url = state.zos_api_url.as_ref()?;

    #[derive(serde::Deserialize)]
    struct ProfileSummary {
        #[serde(rename = "firstName")]
        first_name: Option<String>,
        #[serde(rename = "lastName")]
        last_name: Option<String>,
    }

    #[derive(serde::Deserialize)]
    struct ZosUser {
        #[serde(rename = "profileSummary")]
        profile_summary: Option<ProfileSummary>,
        #[serde(rename = "primaryZID")]
        primary_zid: Option<String>,
    }

    let resp = state
        .http_client
        .get(format!("{}/api/users/current", zos_url))
        .bearer_auth(token)
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let zos_user: ZosUser = resp.json().await.ok()?;

    // Try firstName + lastName first
    if let Some(ref ps) = zos_user.profile_summary {
        let first = ps.first_name.as_deref().unwrap_or("");
        let last = ps.last_name.as_deref().unwrap_or("");
        let full = format!("{} {}", first, last).trim().to_string();
        if !full.is_empty() {
            return Some(full);
        }
    }

    // Fall back to primaryZID
    zos_user.primary_zid.filter(|z| !z.is_empty())
}
