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

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
