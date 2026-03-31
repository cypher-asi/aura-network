use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{ActivityEvent, Comment, CreateActivityEventRequest, CreateCommentRequest};

pub async fn post_activity(
    pool: &PgPool,
    input: &CreateActivityEventRequest,
) -> Result<ActivityEvent, AppError> {
    crate::models::validate_event_type(&input.event_type)?;

    if input.title.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Activity event title must not be empty".into(),
        ));
    }

    let post_type = input.post_type.as_deref().unwrap_or("event");

    let event = sqlx::query_as::<_, ActivityEvent>(
        r#"
        INSERT INTO activity_events (profile_id, org_id, project_id, event_type, post_type, title, summary, metadata, agent_id, user_id, push_id, commit_ids)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING *
        "#,
    )
    .bind(input.profile_id)
    .bind(input.org_id)
    .bind(input.project_id)
    .bind(&input.event_type)
    .bind(post_type)
    .bind(input.title.trim())
    .bind(&input.summary)
    .bind(&input.metadata)
    .bind(input.agent_id)
    .bind(input.user_id)
    .bind(input.push_id)
    .bind(&input.commit_ids)
    .fetch_one(pool)
    .await?;

    Ok(event)
}

/// Get feed with filter:
/// - "my-agents": events from the user's agent profiles
/// - "org": events from the user's org
/// - "following": events from profiles the user follows
/// - "everything" / None: all events
pub async fn get_feed(
    pool: &PgPool,
    user_id: Uuid,
    filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    let events = match filter {
        Some("my-agents") => {
            sqlx::query_as::<_, ActivityEvent>(
                r#"
                SELECT ae.* FROM activity_events ae
                JOIN profiles p ON ae.profile_id = p.id
                JOIN agents a ON p.agent_id = a.id AND p.profile_type = 'agent'
                WHERE a.user_id = $1
                ORDER BY ae.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        Some("org") => {
            sqlx::query_as::<_, ActivityEvent>(
                r#"
                SELECT ae.* FROM activity_events ae
                WHERE ae.org_id IN (
                    SELECT org_id FROM org_members WHERE user_id = $1
                )
                ORDER BY ae.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        Some("following") => {
            sqlx::query_as::<_, ActivityEvent>(
                r#"
                SELECT ae.* FROM activity_events ae
                WHERE ae.profile_id IN (
                    SELECT f.target_profile_id FROM follows f
                    JOIN profiles p ON f.follower_profile_id = p.id
                    WHERE p.user_id = $1 AND p.profile_type = 'user'
                )
                AND (
                    ae.project_id IS NULL
                    OR NOT EXISTS (
                        SELECT 1 FROM projects p
                        WHERE p.id = ae.project_id AND p.visibility = 'private'
                    )
                    OR ae.org_id IN (
                        SELECT org_id FROM org_members WHERE user_id = $1
                    )
                )
                ORDER BY ae.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
        _ => {
            // "everything" — all events, excluding private project activity
            // unless the viewer is a member of the project's org.
            sqlx::query_as::<_, ActivityEvent>(
                r#"
                SELECT ae.* FROM activity_events ae
                WHERE (
                    ae.project_id IS NULL
                    OR NOT EXISTS (
                        SELECT 1 FROM projects p
                        WHERE p.id = ae.project_id AND p.visibility = 'private'
                    )
                    OR ae.org_id IN (
                        SELECT org_id FROM org_members WHERE user_id = $1
                    )
                )
                ORDER BY ae.created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        }
    };

    Ok(events)
}

pub async fn get_profile_activity(
    pool: &PgPool,
    profile_id: Uuid,
    viewer_user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    let events = sqlx::query_as::<_, ActivityEvent>(
        r#"
        SELECT * FROM activity_events
        WHERE profile_id = $1
        AND (
            project_id IS NULL
            OR NOT EXISTS (
                SELECT 1 FROM projects p
                WHERE p.id = activity_events.project_id AND p.visibility = 'private'
            )
            OR org_id IN (
                SELECT org_id FROM org_members WHERE user_id = $4
            )
        )
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(profile_id)
    .bind(limit)
    .bind(offset)
    .bind(viewer_user_id)
    .fetch_all(pool)
    .await?;

    Ok(events)
}

pub async fn get_post_by_id(pool: &PgPool, post_id: Uuid) -> Result<ActivityEvent, AppError> {
    sqlx::query_as::<_, ActivityEvent>("SELECT * FROM activity_events WHERE id = $1")
        .bind(post_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".into()))
}

pub async fn create_comment(
    pool: &PgPool,
    activity_event_id: Uuid,
    profile_id: Uuid,
    input: &CreateCommentRequest,
) -> Result<Comment, AppError> {
    if input.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Comment content must not be empty".into(),
        ));
    }

    // Verify event exists
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM activity_events WHERE id = $1)")
            .bind(activity_event_id)
            .fetch_one(pool)
            .await?;

    if !exists {
        return Err(AppError::NotFound("Activity event not found".into()));
    }

    let comment = sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments (activity_event_id, profile_id, content)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(activity_event_id)
    .bind(profile_id)
    .bind(input.content.trim())
    .fetch_one(pool)
    .await?;

    Ok(comment)
}

pub async fn list_comments(
    pool: &PgPool,
    activity_event_id: Uuid,
) -> Result<Vec<Comment>, AppError> {
    let comments = sqlx::query_as::<_, Comment>(
        "SELECT * FROM comments WHERE activity_event_id = $1 ORDER BY created_at",
    )
    .bind(activity_event_id)
    .fetch_all(pool)
    .await?;

    Ok(comments)
}

pub async fn delete_comment(
    pool: &PgPool,
    comment_id: Uuid,
    profile_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM comments WHERE id = $1 AND profile_id = $2")
        .bind(comment_id)
        .bind(profile_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Comment not found or not owned by you".into(),
        ));
    }

    Ok(())
}
