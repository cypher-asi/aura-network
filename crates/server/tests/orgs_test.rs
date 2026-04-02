mod common;

use serde_json::json;

#[sqlx::test(migrations = "../db/migrations")]
async fn create_org(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let res = app
        .post_authed("/api/orgs", &jwt)
        .body(json!({ "name": "Test Org" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Test Org");
    assert!(body["id"].is_string());
    assert!(body["slug"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_orgs_includes_default_and_created(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // First request creates user + default org ("My Team")
    app.post_authed("/api/orgs", &jwt)
        .body(json!({ "name": "Second Org" }).to_string())
        .send()
        .await
        .unwrap();

    let res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let orgs: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(orgs.len(), 2); // default "My Team" + "Second Org"
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_org(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/orgs", &jwt)
        .body(json!({ "name": "Get Test" }).to_string())
        .send()
        .await
        .unwrap();
    let org: serde_json::Value = create_res.json().await.unwrap();
    let org_id = org["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/orgs/{org_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Get Test");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn update_org(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/orgs", &jwt)
        .body(json!({ "name": "Original" }).to_string())
        .send()
        .await
        .unwrap();
    let org: serde_json::Value = create_res.json().await.unwrap();
    let org_id = org["id"].as_str().unwrap();

    let res = app
        .put_authed(&format!("/api/orgs/{org_id}"), &jwt)
        .body(json!({ "name": "Updated" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Updated");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn delete_org(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let create_res = app
        .post_authed("/api/orgs", &jwt)
        .body(json!({ "name": "To Delete" }).to_string())
        .send()
        .await
        .unwrap();
    let org: serde_json::Value = create_res.json().await.unwrap();
    let org_id = org["id"].as_str().unwrap();

    let res = app
        .delete_authed(&format!("/api/orgs/{org_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 204);

    // Verify it no longer appears in list
    let list_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert!(
        !orgs.iter().any(|o| o["name"] == "To Delete"),
        "Deleted org should not appear in list"
    );
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_members(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // Trigger user creation + default org
    let orgs_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/orgs/{org_id}/members"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let members: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(members.len(), 1); // creator is the only member
    assert_eq!(members[0]["role"], "owner");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn invite_flow_create_list_accept(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt_owner = common::test_jwt("owner-user");
    let jwt_invitee = common::test_jwt("invitee-user");

    // Owner creates default org
    let orgs_res = app
        .get_authed("/api/orgs", &jwt_owner)
        .send()
        .await
        .unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    // Create invite
    let invite_res = app
        .post_authed(&format!("/api/orgs/{org_id}/invites"), &jwt_owner)
        .send()
        .await
        .unwrap();
    assert_eq!(invite_res.status(), 200);
    let invite: serde_json::Value = invite_res.json().await.unwrap();
    let token = invite["token"].as_str().unwrap();
    assert_eq!(invite["status"], "pending");

    // List invites
    let list_res = app
        .get_authed(&format!("/api/orgs/{org_id}/invites"), &jwt_owner)
        .send()
        .await
        .unwrap();
    assert_eq!(list_res.status(), 200);
    let invites: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert_eq!(invites.len(), 1);

    // Invitee accepts
    let accept_res = app
        .post_authed(&format!("/api/invites/{token}/accept"), &jwt_invitee)
        .body(json!({ "displayName": "Invitee" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(accept_res.status(), 200);
    let member: serde_json::Value = accept_res.json().await.unwrap();
    assert_eq!(member["role"], "member");

    // Verify org now has 2 members
    let members_res = app
        .get_authed(&format!("/api/orgs/{org_id}/members"), &jwt_owner)
        .send()
        .await
        .unwrap();
    let members: Vec<serde_json::Value> = members_res.json().await.unwrap();
    assert_eq!(members.len(), 2);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn revoke_invite(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let orgs_res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();
    let orgs: Vec<serde_json::Value> = orgs_res.json().await.unwrap();
    let org_id = orgs[0]["id"].as_str().unwrap();

    // Create invite
    let invite_res = app
        .post_authed(&format!("/api/orgs/{org_id}/invites"), &jwt)
        .send()
        .await
        .unwrap();
    let invite: serde_json::Value = invite_res.json().await.unwrap();
    let invite_id = invite["id"].as_str().unwrap();

    // Revoke it
    let res = app
        .delete_authed(
            &format!("/api/orgs/{org_id}/invites/{invite_id}"),
            &jwt,
        )
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Verify invite is now revoked (list still returns it but with revoked status)
    let list_res = app
        .get_authed(&format!("/api/orgs/{org_id}/invites"), &jwt)
        .send()
        .await
        .unwrap();
    let invites: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert_eq!(invites.len(), 1);
    assert_eq!(invites[0]["status"], "revoked");
}
