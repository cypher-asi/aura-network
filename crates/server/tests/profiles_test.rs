mod common;

use serde_json::json;

#[sqlx::test(migrations = "../db/migrations")]
async fn get_profile_by_id(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // Get user's profile ID
    let me_res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    let profile_id = me["profileId"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/profiles/{profile_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["id"], profile_id);
    assert_eq!(body["profileType"], "user");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_profile_posts_returns_empty_for_new_user(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let me_res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    let profile_id = me["profileId"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/profiles/{profile_id}/posts"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let posts: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(posts.len(), 0);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_agent_profile(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // Create an agent (which auto-creates a profile)
    let agent_res = app
        .post_authed("/api/agents", &jwt)
        .body(json!({ "name": "Profile Agent" }).to_string())
        .send()
        .await
        .unwrap();
    let agent: serde_json::Value = agent_res.json().await.unwrap();
    let agent_id = agent["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/agents/{agent_id}/profile"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["profileType"], "agent");
    assert_eq!(body["displayName"], "Profile Agent");
}
