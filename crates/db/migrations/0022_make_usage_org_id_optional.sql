-- Allow token_usage_daily to be recorded without an org_id.
-- aura-router may not have org context for every request.
ALTER TABLE token_usage_daily ALTER COLUMN org_id DROP NOT NULL;
ALTER TABLE token_usage_daily DROP CONSTRAINT IF EXISTS token_usage_daily_org_id_fkey;

-- Recreate unique index to handle NULL org_id
DROP INDEX IF EXISTS idx_token_usage_unique;
CREATE UNIQUE INDEX idx_token_usage_unique ON token_usage_daily (
    COALESCE(org_id, '00000000-0000-0000-0000-000000000000'),
    user_id,
    COALESCE(agent_id, '00000000-0000-0000-0000-000000000000'),
    model,
    date
);
