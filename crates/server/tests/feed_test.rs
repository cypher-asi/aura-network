mod common;

use serde_json::json;

/// Helper: get the user's profile ID.
async fn get_profile_id(app: &common::TestApp, jwt: &str) -> String {
    let res = app.get_authed("/api/users/me", jwt).send().await.unwrap();
    let me: serde_json::Value = res.json().await.unwrap();
    me["profileId"].as_str().unwrap().to_string()
}

/// Helper: create a post and return its ID.
async fn create_post(app: &common::TestApp, jwt: &str, profile_id: &str) -> serde_json::Value {
    let res = app
        .post_authed("/api/posts", jwt)
        .body(
            json!({
                "profileId": profile_id,
                "eventType": "post",
                "title": "Test Post",
                "summary": "A test post"
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    res.json().await.unwrap()
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_feed_returns_empty_for_new_user(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let res = app.get_authed("/api/feed", &jwt).send().await.unwrap();

    assert_eq!(res.status(), 200);
    let feed: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(feed.len(), 0);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn post_activity(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let profile_id = get_profile_id(&app, &jwt).await;

    let post = create_post(&app, &jwt, &profile_id).await;

    assert_eq!(post["title"], "Test Post");
    assert!(post["id"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_post(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let profile_id = get_profile_id(&app, &jwt).await;

    let post = create_post(&app, &jwt, &profile_id).await;
    let post_id = post["id"].as_str().unwrap();

    let res = app
        .get_authed(&format!("/api/posts/{post_id}"), &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["title"], "Test Post");
}

#[sqlx::test(migrations = "../db/migrations")]
async fn comments_crud(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    let profile_id = get_profile_id(&app, &jwt).await;

    let post = create_post(&app, &jwt, &profile_id).await;
    let post_id = post["id"].as_str().unwrap();

    // Create comment
    let comment_res = app
        .post_authed(&format!("/api/posts/{post_id}/comments"), &jwt)
        .body(json!({ "content": "Nice post!" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(comment_res.status(), 200);
    let comment: serde_json::Value = comment_res.json().await.unwrap();
    let comment_id = comment["id"].as_str().unwrap();
    assert_eq!(comment["content"], "Nice post!");

    // List comments
    let list_res = app
        .get_authed(&format!("/api/posts/{post_id}/comments"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(list_res.status(), 200);
    let comments: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert_eq!(comments.len(), 1);

    // Delete comment
    let del_res = app
        .delete_authed(&format!("/api/comments/{comment_id}"), &jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(del_res.status(), 204);

    // Verify deleted
    let list_after = app
        .get_authed(&format!("/api/posts/{post_id}/comments"), &jwt)
        .send()
        .await
        .unwrap();
    let comments_after: Vec<serde_json::Value> = list_after.json().await.unwrap();
    assert_eq!(comments_after.len(), 0);
}
