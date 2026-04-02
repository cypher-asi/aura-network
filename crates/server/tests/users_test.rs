mod common;

use serde_json::json;

#[sqlx::test(migrations = "../db/migrations")]
async fn get_me_creates_user_on_first_access(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("new-user");

    let res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    // UserResponse flattens User fields + profileId
    assert!(body["id"].is_string());
    assert!(body["profileId"].is_string());
    assert!(body["zeroUserId"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn update_me(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    // Trigger user creation
    app.get_authed("/api/users/me", &jwt).send().await.unwrap();

    let res = app
        .put_authed("/api/users/me", &jwt)
        .body(json!({ "displayName": "New Name", "bio": "Hello" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["displayName"], "New Name");
    assert_eq!(body["bio"], "Hello");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_user_by_id(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let me_res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    let user_id = me["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/users/{user_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["id"], user_id);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_user_profile(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let me_res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    let user_id = me["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/users/{user_id}/profile"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["profileType"], "user");
}
