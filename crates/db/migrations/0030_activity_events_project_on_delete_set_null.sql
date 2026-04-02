-- Replace the plain REFERENCES constraint on activity_events.project_id
-- with ON DELETE SET NULL so deleting a project nullifies the reference
-- rather than leaving a dangling FK.
ALTER TABLE activity_events
    DROP CONSTRAINT IF EXISTS activity_events_project_id_fkey;

ALTER TABLE activity_events
    ADD CONSTRAINT activity_events_project_id_fkey
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL;
