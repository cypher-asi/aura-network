mod common;

use serde_json::json;

/// Helper: get the default org ID.
async fn get_default_org_id(app: &common::TestApp, jwt: &str) -> String {
    let res = app.get_authed("/api/orgs", jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = res.json().await.unwrap();
    orgs[0]["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../db/migrations")]
async fn record_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let res = app
        .post_authed("/api/usage", &jwt)
        .body(
            json!({
                "orgId": org_id,
                "userId": "00000000-0000-0000-0000-000000000000",
                "model": "claude-sonnet-4-6",
                "inputTokens": 100,
                "outputTokens": 50,
                "estimatedCostUsd": 0.005
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 204);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn record_usage_persists_task_id_and_duration(pool: sqlx::PgPool) {
    // The router stamps taskId + durationMs so per-task model time can be
    // computed. Verify the round-trip lands in token_usage_daily and the
    // new /internal/tasks/:id/usage endpoint aggregates it correctly.
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id_str = get_default_org_id(&app, &jwt).await;
    let task_id = uuid::Uuid::new_v4();

    app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let user_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM users ORDER BY created_at DESC LIMIT 1")
            .fetch_one(&app.pool)
            .await
            .unwrap();

    // Two usage rows for the same task: 1000ms + 500ms = 1500ms total.
    for duration in [1000_i64, 500_i64] {
        let res = app
            .post_internal("/internal/usage")
            .body(
                json!({
                    "orgId": org_id_str,
                    "userId": user_id.to_string(),
                    "projectId": null,
                    "agentId": null,
                    "taskId": task_id.to_string(),
                    "model": "claude-test",
                    "inputTokens": 10,
                    "outputTokens": 5,
                    "estimatedCostUsd": 0.0001,
                    "durationMs": duration,
                })
                .to_string(),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 204);
    }

    let res = app
        .get_internal(&format!("/internal/tasks/{task_id}/usage"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["totalInputTokens"], 20);
    assert_eq!(body["totalOutputTokens"], 10);
    assert_eq!(body["totalDurationMs"], 1500);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn project_usage_response_includes_total_duration_ms(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id_str = get_default_org_id(&app, &jwt).await;
    let project_id = uuid::Uuid::new_v4();

    app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let user_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM users ORDER BY created_at DESC LIMIT 1")
            .fetch_one(&app.pool)
            .await
            .unwrap();

    app.post_internal("/internal/usage")
        .body(
            json!({
                "orgId": org_id_str,
                "userId": user_id.to_string(),
                "projectId": project_id.to_string(),
                "model": "m",
                "inputTokens": 1,
                "outputTokens": 1,
                "estimatedCostUsd": 0.0,
                "durationMs": 750,
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    let res = app
        .get_internal(&format!("/internal/projects/{project_id}/usage"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["totalDurationMs"], 750);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_org_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let res = app
        .get_authed(&format!("/api/orgs/{org_id}/usage"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["totalInputTokens"].is_number());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_member_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let res = app
        .get_authed(&format!("/api/orgs/{org_id}/usage/members"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let members: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(members.is_empty() || members[0]["userId"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_personal_usage(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // Trigger user creation
    app.get_authed("/api/users/me", &jwt).send().await.unwrap();

    let res = app
        .get_authed("/api/users/me/usage", &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_stats(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let res = app.get_authed("/api/stats", &jwt).send().await.unwrap();

    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn check_budget(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let res = app
        .get_authed(&format!("/api/orgs/{org_id}/budget"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
}
