mod common;

use serde_json::json;

/// Helper: get the default org ID for a user (created by resolve_user on first request).
async fn get_default_org_id(app: &common::TestApp, jwt: &str) -> String {
    let res = app.get_authed("/api/orgs", jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = res.json().await.unwrap();
    orgs[0]["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../db/migrations")]
async fn create_project(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let res = app
        .post_authed("/api/projects", &jwt)
        .body(
            json!({
                "orgId": org_id,
                "name": "Test Project",
                "description": "A test project"
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Test Project");
    assert_eq!(body["status"], "active");
    assert_eq!(body["visibility"], "private");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_projects(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    app.post_authed("/api/projects", &jwt)
        .body(json!({ "orgId": org_id, "name": "Project A" }).to_string())
        .send()
        .await
        .unwrap();

    let res = app
        .get_authed(&format!("/api/projects?org_id={org_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let projects: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(projects.len(), 1);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_project(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let create_res = app
        .post_authed("/api/projects", &jwt)
        .body(json!({ "orgId": org_id, "name": "Get Test" }).to_string())
        .send()
        .await
        .unwrap();
    let project: serde_json::Value = create_res.json().await.unwrap();
    let project_id = project["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/projects/{project_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Get Test");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn update_project(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let create_res = app
        .post_authed("/api/projects", &jwt)
        .body(json!({ "orgId": org_id, "name": "Original" }).to_string())
        .send()
        .await
        .unwrap();
    let project: serde_json::Value = create_res.json().await.unwrap();
    let project_id = project["id"].as_str().unwrap();

    let res = app
        .put_authed(&format!("/api/projects/{project_id}"), &jwt)
        .body(json!({ "name": "Updated", "status": "active", "visibility": "private" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Updated");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delete_project_cascade_nullifies_activity_events(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let create_res = app
        .post_authed("/api/projects", &jwt)
        .body(json!({ "orgId": org_id, "name": "To Delete" }).to_string())
        .send()
        .await
        .unwrap();
    let project: serde_json::Value = create_res.json().await.unwrap();
    let project_id = project["id"].as_str().unwrap();

    // Delete the project
    let res = app
        .delete_authed(&format!("/api/projects/{project_id}"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Verify it's gone
    let get_res = app
        .get_authed(&format!("/api/projects/{project_id}"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(get_res.status(), 404);
}
