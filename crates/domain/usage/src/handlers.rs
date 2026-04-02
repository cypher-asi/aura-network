use sqlx::PgPool;
use uuid::Uuid;

use aura_network_core::AppError;

use crate::models::{
    BudgetStatus, MemberUsage, PlatformStats, RealTimePlatformStats, RecordUsageRequest,
    UsageSummary,
};
use crate::repo;

pub async fn record_usage(pool: &PgPool, input: RecordUsageRequest) -> Result<(), AppError> {
    repo::record_usage(pool, &input).await
}

pub async fn get_org_usage(
    pool: &PgPool,
    org_id: Uuid,
    period: Option<&str>,
) -> Result<UsageSummary, AppError> {
    repo::get_org_usage(pool, org_id, period).await
}

pub async fn get_member_usage(
    pool: &PgPool,
    org_id: Uuid,
    period: Option<&str>,
) -> Result<Vec<MemberUsage>, AppError> {
    repo::get_member_usage(pool, org_id, period).await
}

pub async fn get_personal_usage(
    pool: &PgPool,
    user_id: Uuid,
    period: Option<&str>,
) -> Result<UsageSummary, AppError> {
    repo::get_personal_usage(pool, user_id, period).await
}

pub async fn get_project_usage(
    pool: &PgPool,
    project_id: Uuid,
    period: Option<&str>,
) -> Result<UsageSummary, AppError> {
    repo::get_project_usage(pool, project_id, period).await
}

pub async fn check_budget(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
) -> Result<BudgetStatus, AppError> {
    repo::check_budget(pool, org_id, user_id).await
}

pub async fn get_platform_stats(pool: &PgPool) -> Result<Option<PlatformStats>, AppError> {
    repo::get_platform_stats(pool).await
}

pub async fn get_realtime_platform_stats(pool: &PgPool) -> Result<RealTimePlatformStats, AppError> {
    repo::get_realtime_platform_stats(pool).await
}
