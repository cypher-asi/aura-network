-- Allow projects to be soft-deleted: status='deleted' is recoverable via the
-- new POST /api/projects/:id/restore endpoint. Hard DELETE is no longer
-- exposed externally; the existing DELETE /api/projects/:id handler now sets
-- status='deleted' so all linked rows (activity_events, token_usage_daily,
-- aura-storage tasks/sessions/specs) stay intact for recovery.
ALTER TABLE projects DROP CONSTRAINT projects_status_check;
ALTER TABLE projects
    ADD CONSTRAINT projects_status_check
    CHECK (status IN ('active', 'archived', 'deleted'));
