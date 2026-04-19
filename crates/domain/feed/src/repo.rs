use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{
    ActivityEvent, Comment, CreateActivityEventRequest, CreateCommentRequest, PublicFeedbackEntry,
    VoteSummary,
};

/// Columns we always SELECT from `activity_events`, plus the computed vote
/// aggregate columns. Keeps the feed-shaped query on one line and ensures
/// non-feedback rows still produce zeros for the aggregate fields.
///
/// `$vid` is the viewer's profile id or NULL. Binding NULL keeps the viewer's
/// vote = "none" without branching the SQL.
const SELECT_EVENT_WITH_VOTES: &str = r#"
    SELECT
        ae.id,
        ae.profile_id,
        ae.org_id,
        ae.project_id,
        ae.event_type,
        ae.post_type,
        ae.title,
        ae.summary,
        ae.metadata,
        ae.agent_id,
        ae.user_id,
        ae.push_id,
        ae.commit_ids,
        ae.created_at,
        (SELECT COUNT(*) FROM comments c WHERE c.activity_event_id = ae.id) AS comment_count,
        COALESCE(v.upvotes, 0) AS upvotes,
        COALESCE(v.downvotes, 0) AS downvotes,
        COALESCE(v.upvotes, 0) - COALESCE(v.downvotes, 0) AS vote_score,
        CASE
            WHEN vv.vote = 1 THEN 'up'
            WHEN vv.vote = -1 THEN 'down'
            ELSE 'none'
        END AS viewer_vote
    FROM activity_events ae
    LEFT JOIN LATERAL (
        SELECT
            SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END)::BIGINT AS upvotes,
            SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END)::BIGINT AS downvotes
        FROM feedback_votes fv
        WHERE fv.activity_event_id = ae.id
    ) v ON TRUE
    LEFT JOIN LATERAL (
        SELECT vote FROM feedback_votes
        WHERE activity_event_id = ae.id AND profile_id = $1
        LIMIT 1
    ) vv ON TRUE
"#;

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

    // profile_id is optional on the wire but required in the DB. The HTTP
    // layer fills it in from the authenticated user's profile when a thin
    // proxy (like aura-os-server) doesn't carry one. If we ever reach here
    // without one, it's a programmer error — surface it as a clean 400.
    let profile_id = input.profile_id.ok_or_else(|| {
        AppError::BadRequest("profileId is required to create an activity event".into())
    })?;

    // Insert the activity event, then re-select it via the vote-joining query
    // so the returned shape is identical to list/get and aggregate fields are
    // present even before any votes exist.
    let inserted_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO activity_events (
            profile_id, org_id, project_id, event_type, post_type, title, summary,
            metadata, agent_id, user_id, push_id, commit_ids
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id
        "#,
    )
    .bind(profile_id)
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

    get_post_by_id(pool, inserted_id, Some(profile_id)).await
}

/// Get feed with filter and viewer-aware vote aggregates.
pub async fn get_feed(
    pool: &PgPool,
    user_id: Uuid,
    viewer_profile_id: Option<Uuid>,
    filter: Option<&str>,
    sort: FeedSort,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    let where_clause = match filter {
        Some("my-agents") => {
            r#"
            JOIN profiles p ON ae.profile_id = p.id
            JOIN agents a ON p.agent_id = a.id AND p.profile_type = 'agent'
            WHERE a.user_id = $2
            "#
        }
        Some("org") => {
            r#"
            WHERE ae.org_id IN (SELECT org_id FROM org_members WHERE user_id = $2)
            "#
        }
        Some("following") => {
            r#"
            WHERE ae.profile_id IN (
                SELECT f.target_profile_id FROM follows f
                JOIN profiles p ON f.follower_profile_id = p.id
                WHERE p.user_id = $2 AND p.profile_type = 'user'
            )
            AND (
                ae.project_id IS NULL
                OR NOT EXISTS (
                    SELECT 1 FROM projects p
                    WHERE p.id = ae.project_id AND p.visibility = 'private'
                )
                OR ae.org_id IN (SELECT org_id FROM org_members WHERE user_id = $2)
            )
            "#
        }
        Some("feedback") => {
            // Feedback is a global surface — anyone authenticated sees all
            // feedback posts regardless of org/following.
            r#"
            WHERE ae.event_type = 'feedback'
            "#
        }
        _ => {
            r#"
            WHERE (
                ae.project_id IS NULL
                OR NOT EXISTS (
                    SELECT 1 FROM projects p
                    WHERE p.id = ae.project_id AND p.visibility = 'private'
                )
                OR ae.org_id IN (SELECT org_id FROM org_members WHERE user_id = $2)
            )
            "#
        }
    };

    let order_by = sort.order_by_sql();
    let query = format!(
        "{SELECT_EVENT_WITH_VOTES}{where_clause} ORDER BY {order_by} LIMIT $3 OFFSET $4"
    );

    let events = sqlx::query_as::<_, ActivityEvent>(&query)
        .bind(viewer_profile_id)
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(events)
}

/// Sorting strategies for the feed. The default `Latest` matches the previous
/// behaviour so no caller needs to change unless they opt into a different
/// mode.
#[derive(Debug, Clone, Copy)]
pub enum FeedSort {
    Latest,
    MostVoted,
    LeastVoted,
    Popular,
    Trending,
}

impl FeedSort {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "latest" => Some(Self::Latest),
            "most_voted" => Some(Self::MostVoted),
            "least_voted" => Some(Self::LeastVoted),
            "popular" => Some(Self::Popular),
            "trending" => Some(Self::Trending),
            _ => None,
        }
    }

    fn order_by_sql(self) -> &'static str {
        match self {
            Self::Latest => "ae.created_at DESC",
            Self::MostVoted => "vote_score DESC, ae.created_at DESC",
            Self::LeastVoted => "vote_score ASC, ae.created_at DESC",
            Self::Popular => {
                "(COALESCE(v.upvotes,0) - COALESCE(v.downvotes,0) + (SELECT COUNT(1) FROM comments c WHERE c.activity_event_id = ae.id)) DESC, ae.created_at DESC"
            }
            // Trending mirrors the phase-1 client formula:
            // (vote_score + comment_count) / (hours_since_created + 2) ^ 1.5
            Self::Trending => {
                "((COALESCE(v.upvotes,0) - COALESCE(v.downvotes,0) + (SELECT COUNT(1) FROM comments c WHERE c.activity_event_id = ae.id))::float8 / POW(EXTRACT(EPOCH FROM (NOW() - ae.created_at))/3600 + 2, 1.5)) DESC, ae.created_at DESC"
            }
        }
    }
}

impl Default for FeedSort {
    fn default() -> Self {
        Self::Latest
    }
}

pub async fn get_profile_activity(
    pool: &PgPool,
    profile_id: Uuid,
    viewer_profile_id: Option<Uuid>,
    viewer_user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityEvent>, AppError> {
    let query = format!(
        r#"{SELECT_EVENT_WITH_VOTES}
        WHERE ae.profile_id = $2
        AND (
            ae.project_id IS NULL
            OR NOT EXISTS (
                SELECT 1 FROM projects p
                WHERE p.id = ae.project_id AND p.visibility = 'private'
            )
            OR ae.org_id IN (SELECT org_id FROM org_members WHERE user_id = $3)
        )
        ORDER BY ae.created_at DESC LIMIT $4 OFFSET $5"#
    );
    let events = sqlx::query_as::<_, ActivityEvent>(&query)
        .bind(viewer_profile_id)
        .bind(profile_id)
        .bind(viewer_user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(events)
}

pub async fn get_post_by_id(
    pool: &PgPool,
    post_id: Uuid,
    viewer_profile_id: Option<Uuid>,
) -> Result<ActivityEvent, AppError> {
    let query = format!("{SELECT_EVENT_WITH_VOTES} WHERE ae.id = $2");
    sqlx::query_as::<_, ActivityEvent>(&query)
        .bind(viewer_profile_id)
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

/// Upsert a viewer's vote on a post. `vote_value` is 1 for up, -1 for down,
/// or `None` to clear any existing vote. Returns the fresh aggregate summary.
pub async fn upsert_vote(
    pool: &PgPool,
    post_id: Uuid,
    profile_id: Uuid,
    vote_value: Option<i16>,
) -> Result<VoteSummary, AppError> {
    // Confirm the target post exists before mutating votes so we can surface a
    // clean 404 instead of a FK error.
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM activity_events WHERE id = $1)")
            .bind(post_id)
            .fetch_one(pool)
            .await?;
    if !exists {
        return Err(AppError::NotFound("Post not found".into()));
    }

    match vote_value {
        None => {
            sqlx::query(
                "DELETE FROM feedback_votes WHERE activity_event_id = $1 AND profile_id = $2",
            )
            .bind(post_id)
            .bind(profile_id)
            .execute(pool)
            .await?;
        }
        Some(v) => {
            sqlx::query(
                r#"
                INSERT INTO feedback_votes (activity_event_id, profile_id, vote)
                VALUES ($1, $2, $3)
                ON CONFLICT (activity_event_id, profile_id)
                DO UPDATE SET vote = EXCLUDED.vote, updated_at = NOW()
                "#,
            )
            .bind(post_id)
            .bind(profile_id)
            .bind(v)
            .execute(pool)
            .await?;
        }
    }

    get_vote_summary(pool, post_id, Some(profile_id)).await
}

pub async fn get_vote_summary(
    pool: &PgPool,
    post_id: Uuid,
    viewer_profile_id: Option<Uuid>,
) -> Result<VoteSummary, AppError> {
    let row = sqlx::query_as::<_, (i64, i64, Option<i16>)>(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN vote = 1 THEN 1 ELSE 0 END), 0)::BIGINT AS upvotes,
            COALESCE(SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END), 0)::BIGINT AS downvotes,
            (
                SELECT vote FROM feedback_votes
                WHERE activity_event_id = $1 AND profile_id = $2
                LIMIT 1
            ) AS viewer
        FROM feedback_votes
        WHERE activity_event_id = $1
        "#,
    )
    .bind(post_id)
    .bind(viewer_profile_id)
    .fetch_one(pool)
    .await?;

    let (upvotes, downvotes, viewer) = row;
    let viewer_vote = match viewer {
        Some(1) => "up",
        Some(-1) => "down",
        _ => "none",
    }
    .to_string();

    Ok(VoteSummary {
        upvotes,
        downvotes,
        score: upvotes - downvotes,
        viewer_vote,
    })
}

/// Shallow-merge `patch` into the post's existing metadata JSON. Keys set to
/// JSON `null` are removed. Returns the updated event in the viewer's frame.
pub async fn patch_metadata(
    pool: &PgPool,
    post_id: Uuid,
    viewer_profile_id: Option<Uuid>,
    patch: &serde_json::Value,
) -> Result<ActivityEvent, AppError> {
    if !patch.is_object() {
        return Err(AppError::BadRequest(
            "metadata patch must be a JSON object".into(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE activity_events
        SET metadata = (
            SELECT jsonb_strip_nulls(
                COALESCE(metadata, '{}'::jsonb) || $2::jsonb
            )
        )
        WHERE id = $1
        "#,
    )
    .bind(post_id)
    .bind(patch)
    .execute(pool)
    .await?;

    get_post_by_id(pool, post_id, viewer_profile_id).await
}

/// Matches a UUID-shaped display_name so we can strip placeholders (users
/// whose real name hasn't been fetched yet from zOS). Kept in sync with the
/// same check that used to live in `aura-web/src/server/feedback.ts`.
const UUID_RE: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

fn looks_like_uuid_display_name(value: &str) -> bool {
    // Hand-rolled check (no regex dep): 36 chars, dashes in the right spots,
    // everything else lowercase hex. Slightly stricter than the old JS regex
    // since we lowercase first, but that's fine — we only want to strip the
    // placeholder form the user layer writes.
    let _ = UUID_RE; // referenced so rustdoc keeps the const accessible.
    if value.len() != 36 {
        return false;
    }
    let bytes = value.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if *b != b'-' {
                    return false;
                }
            }
            _ => {
                let c = b.to_ascii_lowercase();
                let is_hex = c.is_ascii_digit() || (b'a'..=b'f').contains(&c);
                if !is_hex {
                    return false;
                }
            }
        }
    }
    true
}

/// Ordering for the public feedback listing. Mirrors `FeedSort` but keyed by
/// strings so the handler can round-trip the query param verbatim. Unknown
/// values fall through to `latest`.
fn public_order_by_sql(sort: Option<&str>) -> &'static str {
    match sort.unwrap_or("latest") {
        "most_voted" => "vote_score DESC, ae.created_at DESC",
        "least_voted" => "vote_score ASC, ae.created_at DESC",
        "popular" => {
            "(COALESCE(v.upvotes,0) - COALESCE(v.downvotes,0) + COALESCE(cc.comment_count,0)) DESC, ae.created_at DESC"
        }
        "trending" => {
            "((COALESCE(v.upvotes,0) - COALESCE(v.downvotes,0) + COALESCE(cc.comment_count,0))::float8 / POW(EXTRACT(EPOCH FROM (NOW() - ae.created_at))/3600 + 2, 1.5)) DESC, ae.created_at DESC"
        }
        _ => "ae.created_at DESC",
    }
}

/// Unauthenticated read of `event_type = 'feedback'` posts, shaped for
/// marketing / roadmap surfaces. Returns aggregate votes, comment counts, and
/// author profile info directly — no viewer context, so there is no
/// `viewerVote` on the wire.
pub async fn list_public_feedback(
    pool: &PgPool,
    product: &str,
    sort: Option<&str>,
    category: Option<&str>,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<PublicFeedbackEntry>, AppError> {
    let order_by = public_order_by_sql(sort);
    let query = format!(
        r#"
        SELECT
            ae.id,
            ae.title,
            COALESCE(ae.metadata->>'body', ae.summary, '') AS body,
            COALESCE(ae.metadata->>'feedbackCategory', 'feedback') AS category,
            COALESCE(ae.metadata->>'feedbackStatus', 'not_started') AS status,
            COALESCE(v.upvotes, 0)   AS upvotes,
            COALESCE(v.downvotes, 0) AS downvotes,
            COALESCE(v.upvotes, 0) - COALESCE(v.downvotes, 0) AS vote_score,
            COALESCE(cc.comment_count, 0) AS comment_count,
            ae.created_at,
            p.display_name AS author_name,
            p.avatar       AS author_avatar
        FROM activity_events ae
        LEFT JOIN profiles p ON p.id = ae.profile_id
        LEFT JOIN LATERAL (
            SELECT
                SUM(CASE WHEN vote =  1 THEN 1 ELSE 0 END)::BIGINT AS upvotes,
                SUM(CASE WHEN vote = -1 THEN 1 ELSE 0 END)::BIGINT AS downvotes
            FROM feedback_votes fv
            WHERE fv.activity_event_id = ae.id
        ) v ON TRUE
        LEFT JOIN LATERAL (
            SELECT COUNT(1)::BIGINT AS comment_count
            FROM comments c
            WHERE c.activity_event_id = ae.id
        ) cc ON TRUE
        WHERE ae.event_type = 'feedback'
          AND COALESCE(ae.metadata->>'feedbackProduct', 'aura') = $1
          AND ($2::text IS NULL OR ae.metadata->>'feedbackCategory' = $2)
          AND ($3::text IS NULL OR ae.metadata->>'feedbackStatus'   = $3)
        ORDER BY {order_by}
        LIMIT $4
        "#
    );

    let mut rows = sqlx::query_as::<_, PublicFeedbackEntry>(&query)
        .bind(product)
        .bind(category)
        .bind(status)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    for row in rows.iter_mut() {
        if let Some(name) = row.author_name.as_deref() {
            if looks_like_uuid_display_name(name) {
                row.author_name = None;
            }
        }
    }

    Ok(rows)
}
