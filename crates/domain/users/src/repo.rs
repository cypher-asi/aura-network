use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{AppAccessCode, CreateUserFromToken, Profile, UpdateUserRequest, User};

pub async fn upsert_from_token(
    pool: &PgPool,
    input: &CreateUserFromToken,
) -> Result<User, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (zero_user_id, display_name, profile_image, primary_zid)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (zero_user_id) DO UPDATE SET
            profile_image = EXCLUDED.profile_image,
            primary_zid = EXCLUDED.primary_zid,
            last_login_at = NOW(),
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
        ON CONFLICT (user_id) WHERE profile_type = 'user' AND user_id IS NOT NULL DO UPDATE SET
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

pub async fn update(
    pool: &PgPool,
    user_id: Uuid,
    input: &UpdateUserRequest,
) -> Result<User, AppError> {
    if let Some(ref bio) = input.bio {
        if bio.chars().count() > 400 {
            return Err(AppError::BadRequest(
                "Bio must be 400 characters or less".into(),
            ));
        }
    }

    sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET
            display_name = COALESCE($2, display_name),
            bio = COALESCE($3, bio),
            profile_image = COALESCE($4, profile_image),
            location = COALESCE($5, location),
            website = COALESCE($6, website),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&input.display_name)
    .bind(&input.bio)
    .bind(&input.profile_image)
    .bind(&input.location)
    .bind(&input.website)
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

pub async fn update_profile_display_name(
    pool: &PgPool,
    user_id: Uuid,
    display_name: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE profiles SET display_name = $1, updated_at = NOW() WHERE user_id = $2 AND profile_type = 'user'",
    )
    .bind(display_name)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_profile_by_agent_id(pool: &PgPool, agent_id: Uuid) -> Result<Profile, AppError> {
    sqlx::query_as::<_, Profile>(
        "SELECT * FROM profiles WHERE profile_type = 'agent' AND agent_id = $1",
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Profile not found".into()))
}

// ---------------------------------------------------------------------------
// App access codes
// ---------------------------------------------------------------------------

const ACCESS_CODES_PER_USER: i64 = 5;

fn generate_access_code() -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

pub async fn grant_access(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE users SET is_access_granted = true, access_granted_at = NOW(), updated_at = NOW() WHERE id = $1",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn generate_access_codes(pool: &PgPool, user_id: Uuid) -> Result<Vec<AppAccessCode>, AppError> {
    let existing: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM app_access_codes WHERE created_by = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

    let to_create = (ACCESS_CODES_PER_USER - existing).max(0);
    for _ in 0..to_create {
        let code = generate_access_code();
        sqlx::query(
            "INSERT INTO app_access_codes (code, created_by) VALUES ($1, $2) ON CONFLICT (code) DO NOTHING",
        )
        .bind(&code)
        .bind(user_id)
        .execute(pool)
        .await?;
    }

    list_access_codes(pool, user_id).await
}

pub async fn list_access_codes(pool: &PgPool, user_id: Uuid) -> Result<Vec<AppAccessCode>, AppError> {
    let codes = sqlx::query_as::<_, AppAccessCode>(
        "SELECT * FROM app_access_codes WHERE created_by = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(codes)
}

pub async fn redeem_access_code(pool: &PgPool, code: &str, user_id: Uuid) -> Result<AppAccessCode, AppError> {
    // Check if user already has access
    let user = get_by_id(pool, user_id).await?;
    if user.is_access_granted {
        return Err(AppError::BadRequest("You already have access".into()));
    }

    // Find the code
    let access_code = sqlx::query_as::<_, AppAccessCode>(
        "SELECT * FROM app_access_codes WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid access code".into()))?;

    // Check it hasn't been redeemed
    if access_code.status == "redeemed" {
        return Err(AppError::BadRequest("This code has already been used".into()));
    }

    // Prevent self-redemption
    if access_code.created_by == user_id {
        return Err(AppError::BadRequest("You cannot redeem your own access code".into()));
    }

    // Redeem the code
    let redeemed = sqlx::query_as::<_, AppAccessCode>(
        r#"
        UPDATE app_access_codes
        SET status = 'redeemed', redeemed_by = $1, redeemed_at = NOW()
        WHERE id = $2 AND status = 'available'
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(access_code.id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("This code has already been used".into()))?;

    // Grant access to the redeeming user
    grant_access(pool, user_id).await?;

    // Generate codes for the newly granted user
    generate_access_codes(pool, user_id).await?;

    Ok(redeemed)
}
