-- Add machine_type field to agents (local or remote).
ALTER TABLE agents ADD COLUMN machine_type TEXT NOT NULL DEFAULT 'local';
