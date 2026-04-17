-- Feedback votes. One active vote per (post, profile).
-- vote = 1 for upvote, -1 for downvote; clearing a vote deletes the row.

CREATE TABLE feedback_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_event_id UUID NOT NULL REFERENCES activity_events(id) ON DELETE CASCADE,
    profile_id UUID NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (activity_event_id, profile_id)
);

CREATE INDEX idx_feedback_votes_event ON feedback_votes (activity_event_id);
CREATE INDEX idx_feedback_votes_profile ON feedback_votes (profile_id);
