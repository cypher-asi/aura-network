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
