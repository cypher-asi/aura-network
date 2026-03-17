use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use jsonwebtoken::DecodingKey;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize)]
struct JwkKey {
    kid: String,
    n: String,
    e: String,
}

#[derive(Clone)]
pub struct JwksClient {
    jwks_url: String,
    http: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, DecodingKey>>>,
}

impl JwksClient {
    pub fn new(auth0_domain: &str) -> Self {
        Self {
            jwks_url: format!("https://{auth0_domain}/.well-known/jwks.json"),
            http: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_key(&self, kid: &str) -> Result<DecodingKey, String> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(key) = cache.get(kid) {
                return Ok(key.clone());
            }
        }

        // Fetch JWKS
        let resp = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| format!("JWKS fetch failed: {e}"))?;

        let jwks: JwksResponse = resp
            .json()
            .await
            .map_err(|e| format!("JWKS parse failed: {e}"))?;

        // Update cache
        let mut cache = self.cache.write().await;
        for key in &jwks.keys {
            if let Ok(decoding_key) = DecodingKey::from_rsa_components(&key.n, &key.e) {
                cache.insert(key.kid.clone(), decoding_key);
            }
        }

        cache
            .get(kid)
            .cloned()
            .ok_or_else(|| format!("Key ID '{kid}' not found in JWKS"))
    }
}
