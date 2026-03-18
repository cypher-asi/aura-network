use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{Follow, LeaderboardEntry};

pub async fn follow(
    pool: &PgPool,
    follower_profile_id: Uuid,
    target_profile_id: Uuid,
) -> Result<Follow, AppError> {
    if follower_profile_id == target_profile_id {
        return Err(AppError::BadRequest("Cannot follow yourself".into()));
    }

    let follow = sqlx::query_as::<_, Follow>(
        r#"
        INSERT INTO follows (follower_profile_id, target_profile_id)
        VALUES ($1, $2)
        ON CONFLICT (follower_profile_id, target_profile_id) DO NOTHING
        RETURNING *
        "#,
    )
    .bind(follower_profile_id)
    .bind(target_profile_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Conflict("Already following this profile".into()))?;

    Ok(follow)
}

pub async fn unfollow(
    pool: &PgPool,
    follower_profile_id: Uuid,
    target_profile_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM follows WHERE follower_profile_id = $1 AND target_profile_id = $2",
    )
    .bind(follower_profile_id)
    .bind(target_profile_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Not following this profile".into()));
    }

    Ok(())
}

pub async fn list_following(
    pool: &PgPool,
    follower_profile_id: Uuid,
) -> Result<Vec<Follow>, AppError> {
    let follows = sqlx::query_as::<_, Follow>(
        "SELECT * FROM follows WHERE follower_profile_id = $1 ORDER BY created_at DESC",
    )
    .bind(follower_profile_id)
    .fetch_all(pool)
    .await?;

    Ok(follows)
}

pub async fn get_leaderboard(
    pool: &PgPool,
    period: Option<&str>,
    org_id: Option<Uuid>,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>, AppError> {
    let date_filter = match period {
        Some("day") => "AND tud.date = CURRENT_DATE",
        Some("week") => "AND tud.date >= CURRENT_DATE - INTERVAL '7 days'",
        Some("month") => "AND tud.date >= CURRENT_DATE - INTERVAL '30 days'",
        _ => "", // "all" or none — no date filter
    };

    let org_filter = if org_id.is_some() {
        "AND tud.org_id = $2"
    } else {
        ""
    };

    let query = format!(
        r#"
        SELECT
            p.id as profile_id,
            p.display_name,
            p.avatar,
            p.profile_type,
            COALESCE(SUM(tud.input_tokens + tud.output_tokens), 0)::int8 as total_tokens,
            COALESCE(SUM(tud.estimated_cost_usd)::float8, 0.0) as estimated_cost_usd,
            COALESCE(event_counts.event_count, 0)::int8 as event_count
        FROM profiles p
        LEFT JOIN users u ON p.user_id = u.id AND p.profile_type = 'user'
        LEFT JOIN agents a ON p.agent_id = a.id AND p.profile_type = 'agent'
        LEFT JOIN token_usage_daily tud ON (
            (p.profile_type = 'user' AND tud.user_id = u.id)
            OR (p.profile_type = 'agent' AND tud.agent_id = a.id)
        ) {date_filter} {org_filter}
        LEFT JOIN (
            SELECT profile_id, COUNT(*) as event_count
            FROM activity_events
            GROUP BY profile_id
        ) event_counts ON event_counts.profile_id = p.id
        GROUP BY p.id, p.display_name, p.avatar, p.profile_type, event_counts.event_count
        HAVING COALESCE(SUM(tud.input_tokens + tud.output_tokens), 0) > 0
            OR COALESCE(event_counts.event_count, 0) > 0
        ORDER BY total_tokens DESC
        LIMIT $1
        "#,
    );

    let entries = if let Some(oid) = org_id {
        sqlx::query_as::<_, LeaderboardEntry>(&query)
            .bind(limit)
            .bind(oid)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, LeaderboardEntry>(&query)
            .bind(limit)
            .fetch_all(pool)
            .await?
    };

    Ok(entries)
}
