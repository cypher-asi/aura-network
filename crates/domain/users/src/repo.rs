use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{CreateUserFromToken, Profile, UpdateUserRequest, User};

pub async fn upsert_from_token(pool: &PgPool, input: &CreateUserFromToken) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (zero_user_id, display_name, profile_image, primary_zid)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (zero_user_id) DO UPDATE SET
            display_name = EXCLUDED.display_name,
            profile_image = EXCLUDED.profile_image,
            primary_zid = EXCLUDED.primary_zid,
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(&input.zero_user_id)
    .bind(&input.display_name)
    .bind(&input.profile_image)
    .bind(&input.primary_zid)
    .fetch_one(pool)
    .await?;

    // Auto-create user profile if not exists
    sqlx::query(
        r#"
        INSERT INTO profiles (profile_type, user_id, display_name, avatar)
        VALUES ('user', $1, $2, $3)
        ON CONFLICT (user_id) WHERE profile_type = 'user' DO UPDATE SET
            display_name = EXCLUDED.display_name,
            avatar = EXCLUDED.avatar,
            updated_at = NOW()
        "#,
    )
    .bind(user.id)
    .bind(&user.display_name)
    .bind(&user.profile_image)
    .execute(pool)
    .await?;

    Ok(user)
}

pub async fn get_by_id(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn get_by_zero_id(pool: &PgPool, zero_user_id: &str) -> Result<User, AppError> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE zero_user_id = $1")
        .bind(zero_user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn update(pool: &PgPool, user_id: Uuid, input: &UpdateUserRequest) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET
            display_name = COALESCE($2, display_name),
            bio = COALESCE($3, bio),
            profile_image = COALESCE($4, profile_image),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&input.display_name)
    .bind(&input.bio)
    .bind(&input.profile_image)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))
}

pub async fn get_profile(pool: &PgPool, profile_id: Uuid) -> Result<Profile, AppError> {
    sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = $1")
        .bind(profile_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found".into()))
}

pub async fn get_profile_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Profile, AppError> {
    sqlx::query_as::<_, Profile>(
        "SELECT * FROM profiles WHERE profile_type = 'user' AND user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Profile not found".into()))
}
