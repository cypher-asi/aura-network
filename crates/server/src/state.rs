use sqlx::PgPool;

use aura_network_auth::{InternalToken, TokenValidator};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub validator: TokenValidator,
    pub internal_token: InternalToken,
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
