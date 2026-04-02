mod common;

#[sqlx::test(migrations = "../db/migrations")]
async fn valid_jwt_authenticates(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;
    let jwt = common::test_jwt("test-user-id");

    let res = app.get_authed("/api/orgs", &jwt).send().await.unwrap();

    // Should get 200 (empty list), not 401
    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn missing_jwt_returns_401(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;

    let res = app.client.get(app.url("/api/orgs")).send().await.unwrap();

    assert_eq!(res.status(), 401);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn invalid_jwt_returns_401(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;

    let res = app
        .get_authed("/api/orgs", "not-a-valid-jwt")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn valid_internal_token_authenticates(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;

    let res = app
        .get_internal("/internal/usage/network")
        .send()
        .await
        .unwrap();

    // Should get 200, not 401
    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../db/migrations")]
async fn invalid_internal_token_returns_401(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;

    let res = app
        .client
        .get(app.url("/internal/usage/network"))
        .header("X-Internal-Token", "wrong-token")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}
