mod common;

use serde_json::json;

/// Helper: trigger user creation and return their internal user ID.
async fn setup_user(app: &common::TestApp, user_id: &str) -> String {
    let jwt = common::test_jwt(user_id);
    let res = app.get_authed("/api/users/me", &jwt).send().await.unwrap();
    let me: serde_json::Value = res.json().await.unwrap();
    me["id"].as_str().unwrap().to_string()
}

/// Helper: grant access to a user by directly hitting the DB-level flow
/// (create user, then generate codes which requires access).
/// For testing, we use the redeem flow from another user's code.
async fn grant_access_and_get_codes(
    app: &common::TestApp,
    granter_jwt: &str,
) -> Vec<serde_json::Value> {
    let res = app
        .get_authed("/api/access-codes", granter_jwt)
        .send()
        .await
        .unwrap();
    res.json().await.unwrap()
}

#[sqlx::test(migrations = "../db/migrations")]
async fn list_access_codes_empty_for_new_user(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("user-1");
    setup_user(&app, "user-1").await;

    let res = app
        .get_authed("/api/access-codes", &jwt)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let codes: Vec<serde_json::Value> = res.json().await.unwrap();
    // New user without access doesn't have codes yet
    assert_eq!(codes.len(), 0);
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
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid access code"));
}

#[sqlx::test(migrations = "../db/migrations")]
async fn redeem_own_code_blocked(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    let jwt_granter = common::test_jwt("granter");
    setup_user(&app, "granter").await;

    // Insert a code owned by granter (don't grant access, so self-redeem check triggers)
    //
    sqlx::query(
        "INSERT INTO app_access_codes (code, created_by) SELECT 'TESTCODE', id FROM users WHERE zero_user_id = 'granter'",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Granter tries to redeem their own code
    let res = app
        .post_authed("/api/access-codes/redeem", &jwt_granter)
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
async fn redeem_valid_code_grants_access(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    let _jwt_granter = common::test_jwt("granter");
    let jwt_redeemer = common::test_jwt("redeemer");
    setup_user(&app, "granter").await;
    setup_user(&app, "redeemer").await;

    // Grant access to granter and create a code
    sqlx::query("UPDATE users SET is_access_granted = true WHERE zero_user_id = 'granter'")
        .execute(&pool)
        .await
        .unwrap();
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
    assert_eq!(body["status"], "redeemed");

    // Verify redeemer now has access (check user record)
    let me_res = app
        .get_authed("/api/users/me", &jwt_redeemer)
        .send()
        .await
        .unwrap();
    let me: serde_json::Value = me_res.json().await.unwrap();
    assert_eq!(me["isAccessGranted"], true);

    // Verify redeemer got their own 5 codes
    let codes_res = app
        .get_authed("/api/access-codes", &jwt_redeemer)
        .send()
        .await
        .unwrap();
    let codes: Vec<serde_json::Value> = codes_res.json().await.unwrap();
    assert_eq!(codes.len(), 5);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn redeem_already_used_code_returns_error(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool.clone()).await;
    let jwt_redeemer1 = common::test_jwt("redeemer1");
    let jwt_redeemer2 = common::test_jwt("redeemer2");
    setup_user(&app, "redeemer1").await;
    setup_user(&app, "redeemer2").await;

    // Create granter with a code
    setup_user(&app, "granter").await;
    sqlx::query("UPDATE users SET is_access_granted = true WHERE zero_user_id = 'granter'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO app_access_codes (code, created_by) SELECT 'USEDONCE', id FROM users WHERE zero_user_id = 'granter'",
    )
    .execute(&pool)
    .await
    .unwrap();

    // First redemption succeeds
    let res1 = app
        .post_authed("/api/access-codes/redeem", &jwt_redeemer1)
        .body(json!({ "code": "USEDONCE" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res1.status(), 200);

    // Second redemption fails
    let res2 = app
        .post_authed("/api/access-codes/redeem", &jwt_redeemer2)
        .body(json!({ "code": "USEDONCE" }).to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), 400);
    let body: serde_json::Value = res2.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("already been used"));
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
    sqlx::query("UPDATE users SET is_access_granted = true WHERE zero_user_id = 'other'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO app_access_codes (code, created_by) SELECT 'EXTRACD1', id FROM users WHERE zero_user_id = 'other'",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Try to redeem when already has access
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
