-- Add duration tracking to token usage for request timing.
ALTER TABLE token_usage_daily ADD COLUMN duration_ms BIGINT NOT NULL DEFAULT 0;
