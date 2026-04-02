mod common;

use serde_json::json;

/// Helper: get the default org ID.
async fn get_default_org_id(app: &common::TestApp, jwt: &str) -> String {
    let res = app.get_authed("/api/orgs", jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = res.json().await.unwrap();
    orgs[0]["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../db/migrations")]
async fn create_integration(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let res = app
        .post_authed(&format!("/api/orgs/{org_id}/integrations"), &jwt)
        .body(
            json!({
                "integrationType": "github",
                "config": { "owner": "test-org", "repo": "test-repo", "token": "ghp_test" }
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["integrationType"], "github");
    assert!(body["id"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_integrations(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    // Create one first
    app.post_authed(&format!("/api/orgs/{org_id}/integrations"), &jwt)
        .body(
            json!({
                "integrationType": "slack",
                "config": { "webhook": "https://hooks.slack.com/test" }
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    let res = app
        .get_authed(&format!("/api/orgs/{org_id}/integrations"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let integrations: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(integrations.len(), 1);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn update_integration(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let create_res = app
        .post_authed(&format!("/api/orgs/{org_id}/integrations"), &jwt)
        .body(
            json!({
                "integrationType": "github",
                "config": { "owner": "old" }
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    let integration: serde_json::Value = create_res.json().await.unwrap();
    let id = integration["id"].as_str().unwrap();

    let res = app
        .put_authed(&format!("/api/orgs/{org_id}/integrations/{id}"), &jwt)
        .body(json!({ "config": { "owner": "new" } }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["config"]["owner"], "new");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delete_integration(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let create_res = app
        .post_authed(&format!("/api/orgs/{org_id}/integrations"), &jwt)
        .body(
            json!({
                "integrationType": "github",
                "config": {}
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    let integration: serde_json::Value = create_res.json().await.unwrap();
    let id = integration["id"].as_str().unwrap();

    let res = app
        .delete_authed(&format!("/api/orgs/{org_id}/integrations/{id}"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Verify list is empty
    let list_res = app
        .get_authed(&format!("/api/orgs/{org_id}/integrations"), &jwt)
        .send()
        .await
        .unwrap();
    let integrations: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert_eq!(integrations.len(), 0);
}
