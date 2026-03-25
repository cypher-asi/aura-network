-- Add wallet address to agents for on-chain identity.
ALTER TABLE agents ADD COLUMN wallet_address TEXT;
