use sqlx::PgPool;
use tokio::sync::broadcast;

use aura_network_auth::{InternalToken, TokenValidator};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub validator: TokenValidator,
    pub internal_token: InternalToken,
    pub events_tx: broadcast::Sender<String>,
    pub aura_storage_url: Option<String>,
}

impl AsRef<PgPool> for AppState {
    fn as_ref(&self) -> &PgPool {
        &self.pool
    }
}

impl AsRef<TokenValidator> for AppState {
    fn as_ref(&self) -> &TokenValidator {
        &self.validator
    }
}

impl AsRef<InternalToken> for AppState {
    fn as_ref(&self) -> &InternalToken {
        &self.internal_token
    }
}
