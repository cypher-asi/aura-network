use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{
    AccessCodeRedemption, AppAccessCode, CreateUserFromToken, Profile, UpdateUserRequest, User,
};

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

fn generate_code_string() -> String {
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

/// Ensure the user has exactly one access code. Creates one if missing.
pub async fn ensure_access_code(pool: &PgPool, user_id: Uuid) -> Result<AppAccessCode, AppError> {
    if let Some(existing) =
        sqlx::query_as::<_, AppAccessCode>("SELECT * FROM app_access_codes WHERE created_by = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
    {
        return Ok(existing);
    }

    let code = generate_code_string();
    let created = sqlx::query_as::<_, AppAccessCode>(
        "INSERT INTO app_access_codes (code, created_by) VALUES ($1, $2) RETURNING *",
    )
    .bind(&code)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(created)
}

/// Get the user's access code if it exists.
pub async fn get_access_code(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<AppAccessCode>, AppError> {
    let code =
        sqlx::query_as::<_, AppAccessCode>("SELECT * FROM app_access_codes WHERE created_by = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(code)
}

/// Get all redemptions for a code.
pub async fn get_code_redemptions(
    pool: &PgPool,
    code_id: Uuid,
) -> Result<Vec<AccessCodeRedemption>, AppError> {
    let redemptions = sqlx::query_as::<_, AccessCodeRedemption>(
        "SELECT * FROM access_code_redemptions WHERE code_id = $1 ORDER BY redeemed_at",
    )
    .bind(code_id)
    .fetch_all(pool)
    .await?;
    Ok(redemptions)
}

pub async fn redeem_access_code(
    pool: &PgPool,
    code: &str,
    user_id: Uuid,
) -> Result<AppAccessCode, AppError> {
    // Check if user already has access
    let user = get_by_id(pool, user_id).await?;
    if user.is_access_granted {
        return Err(AppError::BadRequest("You already have access".into()));
    }

    // Find the code
    let access_code =
        sqlx::query_as::<_, AppAccessCode>("SELECT * FROM app_access_codes WHERE code = $1")
            .bind(code)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::BadRequest("Invalid access code".into()))?;

    // Check uses remaining
    if access_code.use_count >= access_code.max_uses {
        return Err(AppError::BadRequest(
            "This code has reached its maximum uses".into(),
        ));
    }

    // Prevent self-redemption
    if access_code.created_by == user_id {
        return Err(AppError::BadRequest(
            "You cannot redeem your own access code".into(),
        ));
    }

    // Increment use count
    let updated = sqlx::query_as::<_, AppAccessCode>(
        r#"
        UPDATE app_access_codes
        SET use_count = use_count + 1
        WHERE id = $1 AND use_count < max_uses
        RETURNING *
        "#,
    )
    .bind(access_code.id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("This code has reached its maximum uses".into()))?;

    // Record the redemption
    sqlx::query("INSERT INTO access_code_redemptions (code_id, redeemed_by) VALUES ($1, $2)")
        .bind(access_code.id)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Grant access to the redeeming user
    grant_access(pool, user_id).await?;

    // Generate a code for the newly granted user
    ensure_access_code(pool, user_id).await?;

    Ok(updated)
}
