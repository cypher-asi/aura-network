-- Add visibility field to projects (matches orbit repo visibility pattern).
-- Default 'private' — projects are only visible to org members.
ALTER TABLE projects ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private';
