CREATE TABLE follows (
    follower_profile_id UUID NOT NULL REFERENCES profiles(id),
    target_profile_id UUID NOT NULL REFERENCES profiles(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (follower_profile_id, target_profile_id)
);
