-- Allow profile deletion to cascade to activity events and comments
-- so that deleting an agent (which deletes its profile) doesn't fail
-- when the agent has feed activity.

ALTER TABLE activity_events DROP CONSTRAINT IF EXISTS activity_events_profile_id_fkey;
ALTER TABLE activity_events ADD CONSTRAINT activity_events_profile_id_fkey
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE;

ALTER TABLE comments DROP CONSTRAINT IF EXISTS comments_profile_id_fkey;
ALTER TABLE comments ADD CONSTRAINT comments_profile_id_fkey
    FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE;

ALTER TABLE follows DROP CONSTRAINT IF EXISTS follows_follower_profile_id_fkey;
ALTER TABLE follows ADD CONSTRAINT follows_follower_profile_id_fkey
    FOREIGN KEY (follower_profile_id) REFERENCES profiles(id) ON DELETE CASCADE;

ALTER TABLE follows DROP CONSTRAINT IF EXISTS follows_target_profile_id_fkey;
ALTER TABLE follows ADD CONSTRAINT follows_target_profile_id_fkey
    FOREIGN KEY (target_profile_id) REFERENCES profiles(id) ON DELETE CASCADE;
