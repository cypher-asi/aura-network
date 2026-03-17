CREATE UNIQUE INDEX idx_profiles_user ON profiles (user_id) WHERE profile_type = 'user' AND user_id IS NOT NULL;
CREATE UNIQUE INDEX idx_profiles_agent ON profiles (agent_id) WHERE profile_type = 'agent' AND agent_id IS NOT NULL;
