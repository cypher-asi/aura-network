-- Per-task model-time aggregation. aura-router stamps x-aura-task-id when an
-- LLM call happens during a known task; this column lets aggregation queries
-- (`SUM(duration_ms) GROUP BY task_id`) compute Neo's "model time" stat at
-- task granularity. Nullable + no FK constraint, matching the existing
-- project_id pattern (migration 0026).
ALTER TABLE token_usage_daily ADD COLUMN task_id UUID;

CREATE INDEX idx_token_usage_daily_task_id
    ON token_usage_daily (task_id)
    WHERE task_id IS NOT NULL;

-- Extend the unique index so per-task rows on the same day don't collide
-- with rows that share org/user/agent/project/model but have no task_id.
DROP INDEX idx_token_usage_unique;
CREATE UNIQUE INDEX idx_token_usage_unique ON token_usage_daily (
    COALESCE(org_id, '00000000-0000-0000-0000-000000000000'),
    user_id,
    COALESCE(agent_id, '00000000-0000-0000-0000-000000000000'),
    COALESCE(project_id, '00000000-0000-0000-0000-000000000000'),
    COALESCE(task_id, '00000000-0000-0000-0000-000000000000'),
    model,
    date
);
