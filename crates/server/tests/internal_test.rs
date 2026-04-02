mod common;

use serde_json::json;

/// Helper: create a user via authed request and return their internal user ID and zero_user_id.
async fn setup_user(app: &common::TestApp, zero_user_id: &str) -> (String, String) {
    let jwt = common::test_jwt(zero_user_id);
    let res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = res.json().await.unwrap();
    let internal_id = me["id"].as_str().unwrap().to_string();
    (internal_id, zero_user_id.to_string())
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_user_by_zero_id(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let (internal_id, zero_id) = setup_user(&app, "test-zero-user").await;

    let res = app
        .get_internal(&format!("/internal/users/{zero_id}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["id"], internal_id);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn internal_record_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // Create user + org first
    let orgs_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    let me_res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    let user_id = me["id"].as_str().unwrap();

    let res = app
        .post_internal("/internal/usage")
        .body(
            json!({
                "orgId": org_id,
                "userId": user_id,
                "zeroUserId": "user-1",
                "model": "claude-sonnet-4-6",
                "inputTokens": 200,
                "outputTokens": 100,
                "estimatedCostUsd": 0.01
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 204);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn internal_get_network_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;

    let res = app
        .get_internal("/internal/usage/network")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["totalInputTokens"].is_number());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn internal_get_project_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let orgs_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    // Create a project
    let proj_res = app
        .post_authed("/api/projects", &jwt)
        .body(json!({ "orgId": org_id, "name": "Test" }).to_string())
        .send()
        .await
        .unwrap();
    let project: serde_json::Value = proj_res.json().await.unwrap();
    let project_id = project["id"].as_str().unwrap();

    let res = app
        .get_internal(&format!("/internal/projects/{project_id}/usage"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn internal_get_org_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let orgs_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    let res = app
        .get_internal(&format!("/internal/orgs/{org_id}/usage"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn internal_check_budget(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let me_res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    let user_id = me["id"].as_str().unwrap();

    let orgs_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    let res = app
        .get_internal(&format!("/internal/orgs/{org_id}/members/{user_id}/budget"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn internal_list_org_integrations(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let orgs_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    let res = app
        .get_internal(&format!("/internal/orgs/{org_id}/integrations"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let integrations: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(integrations.is_empty()); // fresh DB, no integrations
}
