CREATE TABLE activity_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES profiles(id),
    org_id UUID REFERENCES organizations(id),
    project_id UUID REFERENCES projects(id),
    event_type TEXT NOT NULL CHECK (event_type IN ('commit', 'task_completed', 'task_failed', 'loop_started', 'loop_finished', 'agent_created')),
    title TEXT NOT NULL,
    summary TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_activity_events_org ON activity_events (org_id, created_at DESC);
CREATE INDEX idx_activity_events_profile ON activity_events (profile_id, created_at DESC);
