mod common;

#[sqlx::test(migrations = "../db/migrations")]
async fn health_returns_200(pool: sqlx::PgPool) {
    let app = common::spawn_app(pool).await;

    let res = app.client.get(app.url("/health")).send().await.unwrap();

    assert_eq!(res.status(), 200);
}
