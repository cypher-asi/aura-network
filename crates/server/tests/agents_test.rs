mod common;

use serde_json::json;

#[sqlx::test(migrations = "../db/migrations")]
async fn create_agent(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let res = app
        .post_authed("/api/agents", &jwt)
        .body(
            json!({
                "name": "Test Agent",
                "role": "developer",
                "personality": "helpful",
                "systemPrompt": "You are a test agent"
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Test Agent");
    assert_eq!(body["role"], "developer");
    assert!(body["id"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_agents(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // Create an agent first
    app.post_authed("/api/agents", &jwt)
        .body(json!({ "name": "Agent A" }).to_string())
        .send()
        .await
        .unwrap();

    let res = app.get_authed("/api/agents", &jwt).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let agents: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["name"], "Agent A");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_agent_as_owner_includes_system_prompt(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/agents", &jwt)
        .body(
            json!({
                "name": "My Agent",
                "systemPrompt": "secret instructions"
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    let agent: serde_json::Value = create_res.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/agents/{agent_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["systemPrompt"], "secret instructions");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_agent_as_non_owner_strips_sensitive_fields(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt_owner = common::test_jwt("owner");
    let jwt_other = common::test_jwt("other-user");

    let create_res = app
        .post_authed("/api/agents", &jwt_owner)
        .body(
            json!({
                "name": "Secret Agent",
                "systemPrompt": "top secret"
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    let agent: serde_json::Value = create_res.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();

    // Different user fetches the agent
    let res = app
        .get_authed(&format!("/api/agents/{agent_id}"), &jwt_other)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(
        body["systemPrompt"].is_null(),
        "system_prompt should be stripped for non-owner"
    );
    assert!(
        body["walletAddress"].is_null(),
        "wallet_address should be stripped for non-owner"
    );
}

#[sqlx::test(migrations = "../db/migrations")]
async fn update_agent(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/agents", &jwt)
        .body(json!({ "name": "Original" }).to_string())
        .send()
        .await
        .unwrap();
    let agent: serde_json::Value = create_res.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();

    let res = app
        .put_authed(&format!("/api/agents/{agent_id}"), &jwt)
        .body(json!({ "name": "Updated" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Updated");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delete_agent(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/agents", &jwt)
        .body(json!({ "name": "To Delete" }).to_string())
        .send()
        .await
        .unwrap();
    let agent: serde_json::Value = create_res.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();

    let res = app
        .delete_authed(&format!("/api/agents/{agent_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 204);

    // Verify it's gone
    let get_res = app
        .get_authed(&format!("/api/agents/{agent_id}"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(get_res.status(), 404);
}

// ---------------------------------------------------------------------------
// Marketplace fields (Phase 3): listing_status / expertise / tags
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../db/migrations")]
async fn create_agent_persists_marketplace_fields(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let res = app
        .post_authed("/api/agents", &jwt)
        .body(
            json!({
                "name": "Hireable Agent",
                "listingStatus": "hireable",
                "expertise": ["coding", "devops"],
                "tags": ["host_mode:harness"],
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["listingStatus"], "hireable");
    assert_eq!(body["expertise"], json!(["coding", "devops"]));
    assert_eq!(body["tags"], json!(["host_mode:harness"]));
    // Server-computed stats start at zero.
    assert_eq!(body["jobs"], 0);
    assert_eq!(body["revenueUsd"], 0.0);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn create_agent_defaults_listing_status_to_closed(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let res = app
        .post_authed("/api/agents", &jwt)
        .body(json!({ "name": "Default Agent" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["listingStatus"], "closed");
    assert_eq!(body["expertise"], json!([]));
    assert_eq!(body["tags"], json!([]));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn update_agent_can_flip_listing_status_to_hireable(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/agents", &jwt)
        .body(json!({ "name": "Flip Me" }).to_string())
        .send()
        .await
        .unwrap();
    let agent: serde_json::Value = create_res.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();
    assert_eq!(agent["listingStatus"], "closed");

    let res = app
        .put_authed(&format!("/api/agents/{agent_id}"), &jwt)
        .body(
            json!({
                "listingStatus": "hireable",
                "expertise": ["coding"],
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["listingStatus"], "hireable");
    assert_eq!(body["expertise"], json!(["coding"]));

    // Re-fetch to confirm persistence (not just the PUT response echo).
    let get_res = app
        .get_authed(&format!("/api/agents/{agent_id}"), &jwt)
        .send()
        .await
        .unwrap();
    let fetched: serde_json::Value = get_res.json().await.unwrap();
    assert_eq!(fetched["listingStatus"], "hireable");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn update_agent_rejects_unknown_listing_status(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/agents", &jwt)
        .body(json!({ "name": "Reject Me" }).to_string())
        .send()
        .await
        .unwrap();
    let agent: serde_json::Value = create_res.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();

    let res = app
        .put_authed(&format!("/api/agents/{agent_id}"), &jwt)
        .body(json!({ "listingStatus": "pending" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_agents_default_remains_caller_scoped(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt_a = common::test_jwt("user-a");
    let jwt_b = common::test_jwt("user-b");

    // User A publishes a hireable agent.
    app.post_authed("/api/agents", &jwt_a)
        .body(
            json!({
                "name": "A's Hireable",
                "listingStatus": "hireable",
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    // User B's unfiltered list MUST still be caller-scoped (empty for B).
    let res = app.get_authed("/api/agents", &jwt_b).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let agents: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(
        agents.is_empty(),
        "default list must be caller-scoped, got {agents:?}"
    );
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_agents_marketplace_filter_returns_cross_user(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt_a = common::test_jwt("user-a");
    let jwt_b = common::test_jwt("user-b");

    // User A publishes one hireable and one closed agent.
    app.post_authed("/api/agents", &jwt_a)
        .body(
            json!({
                "name": "A's Hireable",
                "listingStatus": "hireable",
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    app.post_authed("/api/agents", &jwt_a)
        .body(json!({ "name": "A's Closed" }).to_string())
        .send()
        .await
        .unwrap();

    // User B fetches the marketplace view.
    let res = app
        .get_authed("/api/agents?listing_status=hireable", &jwt_b)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let agents: Vec<serde_json::Value> = res.json().await.unwrap();

    // B sees A's hireable agent but not the closed one.
    let names: Vec<&str> = agents.iter().filter_map(|a| a["name"].as_str()).collect();
    assert_eq!(names, vec!["A's Hireable"]);

    // Sensitive fields must be stripped for non-owners.
    assert!(
        agents[0]["systemPrompt"].is_null(),
        "non-owner must not see systemPrompt in marketplace view"
    );
    assert!(
        agents[0]["walletAddress"].is_null(),
        "non-owner must not see walletAddress in marketplace view"
    );
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_agents_marketplace_filter_supports_expertise(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt_a = common::test_jwt("user-a");
    let jwt_b = common::test_jwt("user-b");

    app.post_authed("/api/agents", &jwt_a)
        .body(
            json!({
                "name": "Coder",
                "listingStatus": "hireable",
                "expertise": ["coding"],
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    app.post_authed("/api/agents", &jwt_a)
        .body(
            json!({
                "name": "Designer",
                "listingStatus": "hireable",
                "expertise": ["ui-ux"],
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    let res = app
        .get_authed("/api/agents?listing_status=hireable&expertise=coding", &jwt_b)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let agents: Vec<serde_json::Value> = res.json().await.unwrap();
    let names: Vec<&str> = agents.iter().filter_map(|a| a["name"].as_str()).collect();
    assert_eq!(names, vec!["Coder"]);
}
