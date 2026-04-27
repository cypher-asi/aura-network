// Shared test helpers — not all items are used by every test file.
#![allow(dead_code)]
use aura_network_auth::{InternalToken, TokenValidator};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::Serialize;
use sqlx::PgPool;
use tokio::sync::broadcast;

use aura_network_server::router::create_router;
use aura_network_server::state::AppState;

const TEST_COOKIE_SECRET: &str = "test-cookie-secret";
const TEST_INTERNAL_TOKEN: &str = "test-internal-token";
const SELF_SIGNED_KID: &str = "jFNXMnFjGrSoDafnLQBohoCNalWcFcTjnKEbkRzWFBHyYJFikdLMHP";

#[allow(dead_code)]
pub struct TestApp {
    pub addr: String,
    pub client: Client,
    pub internal_token: String,
    pub pool: PgPool,
}

#[allow(dead_code)]
impl TestApp {
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.addr, path)
    }

    /// Returns a GET request with Authorization header set.
    pub fn get_authed(&self, path: &str, jwt: &str) -> reqwest::RequestBuilder {
        self.client
            .get(self.url(path))
            .header("Authorization", format!("Bearer {jwt}"))
    }

    /// Returns a POST request with Authorization and Content-Type headers set.
    pub fn post_authed(&self, path: &str, jwt: &str) -> reqwest::RequestBuilder {
        self.client
            .post(self.url(path))
            .header("Authorization", format!("Bearer {jwt}"))
            .header("Content-Type", "application/json")
    }

    /// Returns a PUT request with Authorization and Content-Type headers set.
    pub fn put_authed(&self, path: &str, jwt: &str) -> reqwest::RequestBuilder {
        self.client
            .put(self.url(path))
            .header("Authorization", format!("Bearer {jwt}"))
            .header("Content-Type", "application/json")
    }

    /// Returns a DELETE request with Authorization header set.
    pub fn delete_authed(&self, path: &str, jwt: &str) -> reqwest::RequestBuilder {
        self.client
            .delete(self.url(path))
            .header("Authorization", format!("Bearer {jwt}"))
    }

    /// Returns a GET request with X-Internal-Token header set.
    pub fn get_internal(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(self.url(path))
            .header("X-Internal-Token", &self.internal_token)
    }

    /// Returns a POST request with X-Internal-Token and Content-Type headers set.
    pub fn post_internal(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .post(self.url(path))
            .header("X-Internal-Token", &self.internal_token)
            .header("Content-Type", "application/json")
    }
}

/// Spawn the aura-network server on a random port with a test database.
pub async fn spawn_app(pool: PgPool) -> TestApp {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{port}");

    let (events_tx, _) = broadcast::channel::<String>(16);

    let pool_for_test = pool.clone();
    let state = AppState {
        pool,
        validator: TokenValidator::new(
            "test.auth0.com".to_string(),
            "https://test.api".to_string(),
            TEST_COOKIE_SECRET.to_string(),
        ),
        internal_token: InternalToken(TEST_INTERNAL_TOKEN.to_string()),
        events_tx,
        http_client: Client::new(),
        aura_storage_url: None,
        zos_api_url: None,
        zos_api_internal_token: None,
    };

    let app = create_router().with_state(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    TestApp {
        addr,
        client: Client::new(),
        internal_token: TEST_INTERNAL_TOKEN.to_string(),
        pool: pool_for_test,
    }
}

#[derive(Serialize)]
struct TestClaims {
    id: String,
    sub: String,
    iat: i64,
    exp: i64,
}

/// Generate a valid HS256 JWT for testing, accepted by TokenValidator.
pub fn test_jwt(user_id: &str) -> String {
    let now = chrono::Utc::now().timestamp();
    let claims = TestClaims {
        id: user_id.to_string(),
        sub: format!("auth0|{user_id}"),
        iat: now,
        exp: now + 3600,
    };

    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some(SELF_SIGNED_KID.to_string());

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(TEST_COOKIE_SECRET.as_bytes()),
    )
    .expect("failed to encode test JWT")
}
