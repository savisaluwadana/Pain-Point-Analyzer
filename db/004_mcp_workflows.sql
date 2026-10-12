CREATE TABLE IF NOT EXISTS mcp_api_keys (
 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
 owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 key_hash TEXT NOT NULL UNIQUE,
 label TEXT NOT NULL CHECK (length(label) BETWEEN 1 AND 100),
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 last_used_at TIMESTAMPTZ,
 revoked_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_mcp_api_keys_owner ON mcp_api_keys(owner_id,created_at DESC);

CREATE TABLE IF NOT EXISTS mcp_records (
 owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 60),
 id TEXT NOT NULL CHECK(length(id) BETWEEN 1 AND 200),
 payload JSONB NOT NULL DEFAULT '{}'::jsonb,
 version BIGINT NOT NULL DEFAULT 1,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 PRIMARY KEY(owner_id,kind,id)
);
CREATE INDEX IF NOT EXISTS idx_mcp_records_list ON mcp_records(owner_id,kind,updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_mcp_records_status ON mcp_records(kind,(payload->>'status'),created_at);

-- A persistent, account-scoped research memory. Rows are versioned for optimistic
-- concurrent updates, and a workspace is never fetched without its owner_id.
