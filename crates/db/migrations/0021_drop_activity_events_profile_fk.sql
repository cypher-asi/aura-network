-- Drop FK on profile_id to allow cross-service activity posting.
-- Orbit posts activity events using the user's UUID from JWT,
-- which may not match the profile UUID. Same pattern as other
-- cross-service UUID references.
ALTER TABLE activity_events DROP CONSTRAINT IF EXISTS activity_events_profile_id_fkey;
