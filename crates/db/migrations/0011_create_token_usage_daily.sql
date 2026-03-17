CREATE TABLE token_usage_daily (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id),
    user_id UUID NOT NULL REFERENCES users(id),
    agent_id UUID REFERENCES agents(id),
    model TEXT NOT NULL,
    date DATE NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    estimated_cost_usd NUMERIC(10,4) NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX idx_token_usage_unique ON token_usage_daily (
    org_id, user_id, COALESCE(agent_id, '00000000-0000-0000-0000-000000000000'), model, date
);
CREATE INDEX idx_token_usage_org ON token_usage_daily (org_id, date);
CREATE INDEX idx_token_usage_user ON token_usage_daily (user_id, date);
