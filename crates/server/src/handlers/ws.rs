use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use tokio::sync::broadcast;

use aura_network_core::AppError;

use crate::state::AppState;

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

    // Validate JWT
    let _claims = state
        .validator
        .validate(&token)
        .await
        .map_err(AppError::Unauthorized)?;

    let rx = state.events_tx.subscribe();

    Ok(ws.on_upgrade(|socket| handle_ws(socket, rx)))
}

async fn handle_ws(mut socket: WebSocket, mut rx: broadcast::Receiver<String>) {
    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
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
                    _ => {} // ignore client messages
                }
            }
        }
    }
}
