mod common;

use serde_json::json;

/// Helper: get the user's profile ID.
async fn get_profile_id(app: &common::TestApp, jwt: &str) -> String {
    let res = app.get_authed("/api/users/me", jwt).send().await.unwrap();
    let me: serde_json::Value = res.json().await.unwrap();
    me["profileId"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../db/migrations")]
async fn follow_and_list_following(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt_a = common::test_jwt("user-a");
    let jwt_b = common::test_jwt("user-b");

    let profile_a = get_profile_id(&app, &jwt_a).await;
    let profile_b = get_profile_id(&app, &jwt_b).await;

    // A follows B
    let res = app
        .post_authed("/api/follows", &jwt_a)
        .body(json!({ "targetProfileId": profile_b }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // A's following list includes B
    let list_res = app.get_authed("/api/follows", &jwt_a).send().await.unwrap();
    assert_eq!(list_res.status(), 200);
    let follows: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert_eq!(follows.len(), 1);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn unfollow(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt_a = common::test_jwt("user-a");
    let jwt_b = common::test_jwt("user-b");

    let profile_b = get_profile_id(&app, &jwt_b).await;

    // Follow then unfollow
    app.post_authed("/api/follows", &jwt_a)
        .body(json!({ "targetProfileId": profile_b }).to_string())
        .send()
        .await
        .unwrap();

    let res = app
        .delete_authed(&format!("/api/follows/{profile_b}"), &jwt_a)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Following list is now empty
    let list_res = app.get_authed("/api/follows", &jwt_a).send().await.unwrap();
    let follows: Vec<serde_json::Value> = list_res.json().await.unwrap();
    assert_eq!(follows.len(), 0);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn leaderboard(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");

    let res = app
        .get_authed("/api/leaderboard", &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let entries: Vec<serde_json::Value> = res.json().await.unwrap();
    // Empty leaderboard for fresh DB is fine
    assert!(entries.is_empty() || entries[0]["profileId"].is_string());
}
