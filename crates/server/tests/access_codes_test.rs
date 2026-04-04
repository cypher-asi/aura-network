mod common;

use serde_json::json;

/// Helper: trigger user creation and return their internal user ID.
async fn setup_user(app: &common::TestApp, user_id: &str) -> String {
    let jwt = common::test_jwt(user_id);
    let res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = res.json().await.unwrap();
    me["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_code_returns_none_for_user_without_access(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    setup_user(&app, "user-1").await;

    let res = app
        .get_authed("/api/access-codes", &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body.is_null());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn get_code_auto_generates_for_granted_user(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    let jwt = common::test_jwt("user-1");
    setup_user(&app, "user-1").await;

    // Grant access directly
    sqlx::query("UPDATE users SET is_access_granted = true WHERE zero_user_id = 'user-1'")
        .execute(&pool)
        .await
        .unwrap();

    let res = app
        .get_authed("/api/access-codes", &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["code"].is_string());
    assert_eq!(body["maxUses"], 5);
    assert_eq!(body["useCount"], 0);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn redeem_invalid_code_returns_error(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    setup_user(&app, "user-1").await;

    let res = app
        .post_authed("/api/access-codes/redeem", &jwt)
        .body(json!({ "code": "INVALID1" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn redeem_own_code_blocked(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    let jwt = common::test_jwt("granter");
    setup_user(&app, "granter").await;

    // Insert a code owned by granter
    sqlx::query(
        "INSERT INTO app_access_codes (code, created_by) SELECT 'TESTCODE', id FROM users WHERE zero_user_id = 'granter'",
    )
    .execute(&pool)
    .await
    .unwrap();

    let res = app
        .post_authed("/api/access-codes/redeem", &jwt)
        .body(json!({ "code": "TESTCODE" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("cannot redeem your own"));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn redeem_valid_code_grants_access_and_generates_code(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    let jwt_redeemer = common::test_jwt("redeemer");
    setup_user(&app, "granter").await;
    setup_user(&app, "redeemer").await;

    // Create a code for the granter
    sqlx::query(
        "INSERT INTO app_access_codes (code, created_by) SELECT 'VALIDCD1', id FROM users WHERE zero_user_id = 'granter'",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Redeemer uses the code
    let res = app
        .post_authed("/api/access-codes/redeem", &jwt_redeemer)
        .body(json!({ "code": "VALIDCD1" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["useCount"], 1);

    // Verify redeemer now has access
    let me_res = app
        .get_authed("/api/users/me", &jwt_redeemer)
        .send()
        .await
        .unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    assert_eq!(me["isAccessGranted"], true);

    // Verify redeemer got their own code
    let codes_res = app
        .get_authed("/api/access-codes", &jwt_redeemer)
        .send()
        .await
        .unwrap();
    let code: serde_json::Value = codes_res.json().await.unwrap();
    assert!(code["code"].is_string());
}

#[sqlx::test(migrations = "../db/migrations")]
async fn code_exhausted_after_max_uses(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    setup_user(&app, "granter").await;

    // Create a code with max_uses = 2 for easier testing
    sqlx::query(
        "INSERT INTO app_access_codes (code, created_by, max_uses) SELECT 'LIMITED1', id, 2 FROM users WHERE zero_user_id = 'granter'",
    )
    .execute(&pool)
    .await
    .unwrap();

    // First redemption
    setup_user(&app, "user-1").await;
    let jwt1 = common::test_jwt("user-1");
    let res1 = app
        .post_authed("/api/access-codes/redeem", &jwt1)
        .body(json!({ "code": "LIMITED1" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res1.status(), 200);

    // Second redemption
    setup_user(&app, "user-2").await;
    let jwt2 = common::test_jwt("user-2");
    let res2 = app
        .post_authed("/api/access-codes/redeem", &jwt2)
        .body(json!({ "code": "LIMITED1" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), 200);

    // Third redemption — should fail
    setup_user(&app, "user-3").await;
    let jwt3 = common::test_jwt("user-3");
    let res3 = app
        .post_authed("/api/access-codes/redeem", &jwt3)
        .body(json!({ "code": "LIMITED1" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res3.status(), 400);
    let body: serde_json::Value = res3.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("maximum uses"));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn already_has_access_returns_error(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    let jwt = common::test_jwt("user-1");
    setup_user(&app, "user-1").await;

    // Grant access directly
    sqlx::query("UPDATE users SET is_access_granted = true WHERE zero_user_id = 'user-1'")
        .execute(&pool)
        .await
        .unwrap();

    // Create a code from someone else
    setup_user(&app, "other").await;
    sqlx::query(
        "INSERT INTO app_access_codes (code, created_by) SELECT 'EXTRACD1', id FROM users WHERE zero_user_id = 'other'",
    )
    .execute(&pool)
    .await
    .unwrap();

    let res = app
        .post_authed("/api/access-codes/redeem", &jwt)
        .body(json!({ "code": "EXTRACD1" }).to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("already have access"));
}
