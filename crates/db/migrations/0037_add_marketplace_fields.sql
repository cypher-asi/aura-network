-- Phase 3 marketplace columns. aura-os-server has been writing these
-- typed fields on PUT /api/agents and querying them on
-- GET /api/agents?listing_status=hireable; until now they had no
-- backing schema so saves silently dropped and the marketplace was
-- always empty.
--
-- All columns get safe defaults so existing rows remain valid without
-- a backfill. `jobs` / `revenue_usd` / `reputation` are aggregated
-- stats that a separate worker will populate later; for now they
-- start at 0. `tags` mirrors the legacy dual-write fallback used by
-- aura-os-server's `merge_marketplace_tags`.
ALTER TABLE agents
    ADD COLUMN listing_status TEXT NOT NULL DEFAULT 'closed',
    ADD COLUMN expertise TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN jobs BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN revenue_usd DOUBLE PRECISION NOT NULL DEFAULT 0,
    ADD COLUMN reputation REAL NOT NULL DEFAULT 0,
    ADD COLUMN tags TEXT[] NOT NULL DEFAULT '{}';

ALTER TABLE agents
    ADD CONSTRAINT agents_listing_status_check
    CHECK (listing_status IN ('closed', 'hireable'));

-- Partial index supporting the marketplace's primary query
-- (`SELECT ... WHERE listing_status = 'hireable'`). Closed listings
-- vastly outnumber hireable ones, so the partial form is much smaller
-- and faster than a full index on the column.
CREATE INDEX agents_listing_status_idx ON agents (listing_status)
    WHERE listing_status = 'hireable';
