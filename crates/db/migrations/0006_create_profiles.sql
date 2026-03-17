CREATE TABLE profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_type TEXT NOT NULL CHECK (profile_type IN ('user', 'agent')),
    user_id UUID REFERENCES users(id),
    agent_id UUID REFERENCES agents(id),
    display_name TEXT NOT NULL,
    bio TEXT,
    avatar TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (profile_type = 'user' AND user_id IS NOT NULL AND agent_id IS NULL) OR
        (profile_type = 'agent' AND agent_id IS NOT NULL AND user_id IS NULL)
    )
);
