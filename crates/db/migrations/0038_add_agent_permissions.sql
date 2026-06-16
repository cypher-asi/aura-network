-- Persist the agent capability/scope bundle that Aura OS sends on
-- POST/PUT /api/agents. Without this column aura-network silently
-- dropped `permissions`, forcing clients to rely on local shadows.
ALTER TABLE agents
    ADD COLUMN permissions JSONB NOT NULL DEFAULT '{
        "scope": {
            "orgs": [],
            "projects": [],
            "agent_ids": []
        },
        "capabilities": []
    }'::jsonb;
