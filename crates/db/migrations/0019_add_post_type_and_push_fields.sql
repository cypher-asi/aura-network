-- Add post_type to distinguish generic posts, push posts, and existing events.
-- Add agent_id and user_id for agent+user pairing on all posts.
-- Add push_id and commit_ids for push-type posts from orbit.

ALTER TABLE activity_events ADD COLUMN post_type TEXT NOT NULL DEFAULT 'event';
ALTER TABLE activity_events ADD COLUMN agent_id UUID;
ALTER TABLE activity_events ADD COLUMN user_id UUID;
ALTER TABLE activity_events ADD COLUMN push_id UUID;
ALTER TABLE activity_events ADD COLUMN commit_ids JSONB;
