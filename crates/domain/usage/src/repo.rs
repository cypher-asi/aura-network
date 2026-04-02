use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{
    BudgetStatus, MemberUsage, PlatformStats, RealTimePlatformStats, RecordUsageRequest,
    UsageSummary,
};

fn period_to_date_clause(period: Option<&str>) -> &'static str {
    match period {
        Some("day") => "AND date = CURRENT_DATE",
        Some("week") => "AND date >= CURRENT_DATE - INTERVAL '7 days'",
        Some("month") => "AND date >= CURRENT_DATE - INTERVAL '30 days'",
        _ => "",
    }
}

pub async fn record_usage(pool: &PgPool, input: &RecordUsageRequest) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO token_usage_daily (org_id, user_id, agent_id, project_id, model, date, input_tokens, output_tokens, estimated_cost_usd, duration_ms)
        VALUES ($1, $2, $3, $4, $5, CURRENT_DATE, $6, $7, $8, $9)
        ON CONFLICT (COALESCE(org_id, '00000000-0000-0000-0000-000000000000'), user_id, COALESCE(agent_id, '00000000-0000-0000-0000-000000000000'), COALESCE(project_id, '00000000-0000-0000-0000-000000000000'), model, date)
        DO UPDATE SET
            input_tokens = token_usage_daily.input_tokens + EXCLUDED.input_tokens,
            output_tokens = token_usage_daily.output_tokens + EXCLUDED.output_tokens,
            estimated_cost_usd = token_usage_daily.estimated_cost_usd + EXCLUDED.estimated_cost_usd,
            duration_ms = token_usage_daily.duration_ms + EXCLUDED.duration_ms
        "#,
    )
    .bind(input.org_id)
    .bind(input.user_id)
    .bind(input.agent_id)
    .bind(input.project_id)
    .bind(&input.model)
    .bind(input.input_tokens)
    .bind(input.output_tokens)
    .bind(input.estimated_cost_usd)
    .bind(input.duration_ms.unwrap_or(0))
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_org_usage(
    pool: &PgPool,
    org_id: Uuid,
    period: Option<&str>,
) -> Result<UsageSummary, AppError> {
    let date_clause = period_to_date_clause(period);

    let query = format!(
        r#"
        SELECT
            COALESCE(SUM(input_tokens), 0)::int8 as total_input_tokens,
            COALESCE(SUM(output_tokens), 0)::int8 as total_output_tokens,
            COALESCE(SUM(input_tokens + output_tokens), 0)::int8 as total_tokens,
            COALESCE(SUM(estimated_cost_usd)::float8, 0.0) as total_cost_usd
        FROM token_usage_daily
        WHERE org_id = $1 {date_clause}
        "#,
    );

    let row = sqlx::query_as::<_, UsageSummary>(&query)
        .bind(org_id)
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub async fn get_member_usage(
    pool: &PgPool,
    org_id: Uuid,
    period: Option<&str>,
) -> Result<Vec<MemberUsage>, AppError> {
    let date_clause = period_to_date_clause(period);

    let query = format!(
        r#"
        SELECT
            user_id,
            COALESCE(SUM(input_tokens), 0)::int8 as total_input_tokens,
            COALESCE(SUM(output_tokens), 0)::int8 as total_output_tokens,
            COALESCE(SUM(input_tokens + output_tokens), 0)::int8 as total_tokens,
            COALESCE(SUM(estimated_cost_usd)::float8, 0.0) as total_cost_usd
        FROM token_usage_daily
        WHERE org_id = $1 {date_clause}
        GROUP BY user_id
        ORDER BY total_tokens DESC
        "#,
    );

    let rows = sqlx::query_as::<_, MemberUsage>(&query)
        .bind(org_id)
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub async fn get_personal_usage(
    pool: &PgPool,
    user_id: Uuid,
    period: Option<&str>,
) -> Result<UsageSummary, AppError> {
    let date_clause = period_to_date_clause(period);

    let query = format!(
        r#"
        SELECT
            COALESCE(SUM(input_tokens), 0)::int8 as total_input_tokens,
            COALESCE(SUM(output_tokens), 0)::int8 as total_output_tokens,
            COALESCE(SUM(input_tokens + output_tokens), 0)::int8 as total_tokens,
            COALESCE(SUM(estimated_cost_usd)::float8, 0.0) as total_cost_usd
        FROM token_usage_daily
        WHERE user_id = $1 {date_clause}
        "#,
    );

    let row = sqlx::query_as::<_, UsageSummary>(&query)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub async fn get_project_usage(
    pool: &PgPool,
    project_id: Uuid,
    period: Option<&str>,
) -> Result<UsageSummary, AppError> {
    let date_clause = period_to_date_clause(period);

    let query = format!(
        r#"
        SELECT
            COALESCE(SUM(input_tokens), 0)::int8 as total_input_tokens,
            COALESCE(SUM(output_tokens), 0)::int8 as total_output_tokens,
            COALESCE(SUM(input_tokens + output_tokens), 0)::int8 as total_tokens,
            COALESCE(SUM(estimated_cost_usd)::float8, 0.0) as total_cost_usd
        FROM token_usage_daily
        WHERE project_id = $1 {date_clause}
        "#,
    );

    let row = sqlx::query_as::<_, UsageSummary>(&query)
        .bind(project_id)
        .fetch_one(pool)
        .await?;

    Ok(row)
}

pub async fn check_budget(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<BudgetStatus, AppError> {
    // Get the member's credit budget
    let budget: Option<i64> = sqlx::query_scalar(
        "SELECT credit_budget FROM org_members WHERE org_id = $1 AND user_id = $2",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    // Get current month's usage
    let used: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(input_tokens + output_tokens), 0)::int8
        FROM token_usage_daily
        WHERE org_id = $1 AND user_id = $2
        AND date >= DATE_TRUNC('month', CURRENT_DATE)
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let (allowed, remaining) = match budget {
        Some(b) => (used < b, Some(b - used)),
        None => (true, None), // unlimited
    };

    Ok(BudgetStatus {
        allowed,
        budget,
        used,
        remaining,
    })
}

pub async fn get_platform_stats(pool: &PgPool) -> Result<Option<PlatformStats>, AppError> {
    let stats = sqlx::query_as::<_, PlatformStats>(
        r#"
        SELECT id, date, daily_active_users, total_users, new_signups, projects_created,
               total_input_tokens, total_output_tokens, total_revenue_usd::float8 as total_revenue_usd,
               created_at
        FROM platform_stats ORDER BY date DESC LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(stats)
}

pub async fn get_realtime_platform_stats(pool: &PgPool) -> Result<RealTimePlatformStats, AppError> {
    let stats = sqlx::query_as::<_, RealTimePlatformStats>(
        r#"
        SELECT
            COALESCE((SELECT COUNT(DISTINCT user_id) FROM token_usage_daily WHERE date = CURRENT_DATE), 0)::int8 as daily_active_users,
            COALESCE((SELECT COUNT(*) FROM users), 0)::int8 as total_users,
            COALESCE((SELECT COUNT(*) FROM users WHERE created_at::date = CURRENT_DATE), 0)::int8 as new_signups_today,
            COALESCE((SELECT COUNT(*) FROM projects), 0)::int8 as total_projects,
            COALESCE((SELECT SUM(input_tokens) FROM token_usage_daily), 0)::int8 as total_input_tokens,
            COALESCE((SELECT SUM(output_tokens) FROM token_usage_daily), 0)::int8 as total_output_tokens,
            COALESCE((SELECT SUM(input_tokens + output_tokens) FROM token_usage_daily), 0)::int8 as total_tokens,
            COALESCE((SELECT SUM(estimated_cost_usd)::float8 FROM token_usage_daily), 0.0) as total_cost_usd
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(stats)
}
