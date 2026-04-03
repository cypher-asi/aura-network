-- App-level access codes for early access gating.
-- Zero Pro users bypass this. Non-Pro users redeem a code for access.
-- Each user who gains access gets 5 codes to share.

ALTER TABLE users ADD COLUMN is_access_granted BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN access_granted_at TIMESTAMPTZ;

CREATE TABLE app_access_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code TEXT UNIQUE NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    redeemed_by UUID REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'available' CHECK (status IN ('available', 'redeemed')),
    redeemed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_access_codes_code ON app_access_codes (code);
CREATE INDEX idx_access_codes_created_by ON app_access_codes (created_by);
