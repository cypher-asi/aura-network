-- Add project_id to token usage for per-project cost tracking.
ALTER TABLE token_usage_daily ADD COLUMN project_id UUID;

-- Drop old unique index and recreate with project_id included.
DROP INDEX idx_token_usage_unique;
CREATE UNIQUE INDEX idx_token_usage_unique ON token_usage_daily (
    COALESCE(org_id, '00000000-0000-0000-0000-000000000000'),
    user_id,
    COALESCE(agent_id, '00000000-0000-0000-0000-000000000000'),
    COALESCE(project_id, '00000000-0000-0000-0000-000000000000'),
    model,
    date
);
