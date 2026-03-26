-- Track last login time for users and last active time for agents.
ALTER TABLE users ADD COLUMN last_login_at TIMESTAMPTZ;
ALTER TABLE agents ADD COLUMN last_active_at TIMESTAMPTZ;
