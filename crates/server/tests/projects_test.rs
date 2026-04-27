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
async fn delete_project_soft_deletes(pool: sqlx::PgPool) {
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

    // GET still resolves so the recovery UI can show project details before
    // restore. Status flipped to 'deleted'.
    let get_res = app
        .get_authed(&format!("/api/projects/{project_id}"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(get_res.status(), 200);
    let body: serde_json::Value = get_res.json().await.unwrap();
    assert_eq!(body["status"], "deleted");

    // Excluded from regular project list.
    let list_res = app
        .get_authed(&format!("/api/projects?org_id={org_id}"), &jwt)
        .send()
        .await
        .unwrap();
    let listed: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert!(
        listed.iter().all(|p| p["id"] != project_id),
        "deleted project must not appear in /api/projects list"
    );

    // Visible in deleted-only list.
    let deleted_res = app
        .get_authed(&format!("/api/projects/deleted?org_id={org_id}"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(deleted_res.status(), 200);
    let deleted_list: Vec<serde_json::Value> = deleted_res.json().await.unwrap();
    assert!(
        deleted_list.iter().any(|p| p["id"] == project_id),
        "deleted project must appear in /api/projects/deleted list"
    );
}

#[sqlx::test(migrations = "../db/migrations")]
async fn restore_project_flips_back_to_active(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id = get_default_org_id(&app, &jwt).await;

    let create_res = app
        .post_authed("/api/projects", &jwt)
        .body(json!({ "orgId": org_id, "name": "To Restore" }).to_string())
        .send()
        .await
        .unwrap();
    let project: serde_json::Value = create_res.json().await.unwrap();
    let project_id = project["id"].as_str().unwrap();

    app.delete_authed(&format!("/api/projects/{project_id}"), &jwt)
        .send()
        .await
        .unwrap();

    let res = app
        .post_authed(&format!("/api/projects/{project_id}/restore"), &jwt)
        .body("{}".to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "active");

    // Re-appears in regular list, gone from deleted list.
    let list_res = app
        .get_authed(&format!("/api/projects?org_id={org_id}"), &jwt)
        .send()
        .await
        .unwrap();
    let listed: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert!(listed.iter().any(|p| p["id"] == project_id));

    let deleted_res = app
        .get_authed(&format!("/api/projects/deleted?org_id={org_id}"), &jwt)
        .send()
        .await
        .unwrap();
    let deleted_list: Vec<serde_json::Value> = deleted_res.json().await.unwrap();
    assert!(deleted_list.iter().all(|p| p["id"] != project_id));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delete_project_preserves_associations(pool: sqlx::PgPool) {
    // Soft delete must not nullify or remove rows in linked tables. This is
    // the contract Neo asked for: associations stay intact so restore is a
    // single status flip.
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let org_id_str = get_default_org_id(&app, &jwt).await;
    let org_id: uuid::Uuid = org_id_str.parse().unwrap();

    let create_res = app
        .post_authed("/api/projects", &jwt)
        .body(json!({ "orgId": org_id_str, "name": "Linked" }).to_string())
        .send()
        .await
        .unwrap();
    let project: serde_json::Value = create_res.json().await.unwrap();
    let project_id: uuid::Uuid = project["id"].as_str().unwrap().parse().unwrap();

    // Seed a token_usage_daily row tagged to the project. Use a placeholder
    // user_id since this row is for FK-presence checking only.
    let user_id: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM users ORDER BY created_at DESC LIMIT 1")
            .fetch_one(&app.pool)
            .await
            .unwrap();
    sqlx::query(
        r#"
        INSERT INTO token_usage_daily (org_id, user_id, project_id, model, date, input_tokens, output_tokens, estimated_cost_usd, duration_ms)
        VALUES ($1, $2, $3, 'test-model', CURRENT_DATE, 100, 50, 0.001, 200)
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .bind(project_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Soft delete.
    app.delete_authed(&format!("/api/projects/{project_id}"), &jwt)
        .send()
        .await
        .unwrap();

    // Usage row's project_id must still point at our project.
    let cnt: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM token_usage_daily WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(cnt, 1, "soft delete must not null out token_usage_daily.project_id");
}
