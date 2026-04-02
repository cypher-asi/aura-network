use std::collections::HashSet;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use tokio::sync::broadcast;

use aura_network_core::AppError;

use crate::state::AppState;

const PING_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

pub async fn ws_events(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    let token = query
        .token
        .ok_or_else(|| AppError::Unauthorized("Missing token query parameter".into()))?;

    let claims = state
        .validator
        .validate(&token)
        .await
        .map_err(AppError::Unauthorized)?;

    let user_id = claims
        .user_id()
        .ok_or_else(|| AppError::Unauthorized("Missing user ID in token".into()))?
        .to_string();

    // Resolve the user's org memberships for event filtering
    let org_ids: HashSet<String> =
        if let Ok(user) = aura_network_users::repo::get_by_zero_id(&state.pool, &user_id).await {
            aura_network_orgs::repo::list_for_user(&state.pool, user.id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|org| org.id.to_string())
                .collect()
        } else {
            HashSet::new()
        };

    let rx = state.events_tx.subscribe();

    Ok(ws.on_upgrade(|socket| handle_ws(socket, rx, org_ids)))
}

async fn handle_ws(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<String>,
    user_org_ids: HashSet<String>,
) {
    let mut ping_interval = tokio::time::interval(PING_INTERVAL);

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        // Filter: only forward events the user should see
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg) {
                            let event_org = parsed
                                .pointer("/data/orgId")
                                .and_then(|v| v.as_str());
                            // Allow events with no org (public) or events from user's orgs
                            if let Some(org_id) = event_org {
                                if !user_org_ids.contains(org_id) {
                                    continue;
                                }
                            }
                        }
                        if socket.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "WebSocket client lagged, skipped events");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Pong(_))) => {} // keepalive response received
                    _ => {} // ignore other client messages
                }
            }
            _ = ping_interval.tick() => {
                if socket.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        }
    }
}
