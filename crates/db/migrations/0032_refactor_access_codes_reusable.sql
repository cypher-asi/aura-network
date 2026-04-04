-- Refactor access codes from 5 single-use codes to 1 reusable code per user.
-- Each code can be used up to max_uses times (default 5).

-- Add reusable fields
ALTER TABLE app_access_codes ADD COLUMN max_uses INT NOT NULL DEFAULT 5;
ALTER TABLE app_access_codes ADD COLUMN use_count INT NOT NULL DEFAULT 0;

-- Drop single-use fields
ALTER TABLE app_access_codes DROP COLUMN redeemed_by;
ALTER TABLE app_access_codes DROP COLUMN redeemed_at;
ALTER TABLE app_access_codes DROP COLUMN status;

-- Track each individual redemption
CREATE TABLE access_code_redemptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code_id UUID NOT NULL REFERENCES app_access_codes(id),
    redeemed_by UUID NOT NULL REFERENCES users(id),
    redeemed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_code_redemptions_code ON access_code_redemptions (code_id);
CREATE INDEX idx_code_redemptions_user ON access_code_redemptions (redeemed_by);
