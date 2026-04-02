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

    let res = app
        .get_authed("/api/stats", &jwt)
        .send()
        .await
        .unwrap();

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
