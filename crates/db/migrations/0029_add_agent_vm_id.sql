-- Add VM ID to agents for tracking which VM is running the agent.
ALTER TABLE agents ADD COLUMN vm_id TEXT;
