CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zero_user_id TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    profile_image TEXT,
    primary_zid TEXT,
    bio TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
