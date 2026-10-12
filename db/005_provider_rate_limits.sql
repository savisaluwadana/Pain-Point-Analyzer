-- Conservative application-side budgets; Reddit's own quota headers and terms remain authoritative.
-- Global budget uses owner_id NULL; per-user rows cascade when an account is deleted.
CREATE TABLE IF NOT EXISTS reddit_api_budgets (
 scope TEXT PRIMARY KEY,
 owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
 window_started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 request_count INTEGER NOT NULL DEFAULT 0 CHECK (request_count >= 0),
 blocked_until TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_reddit_api_budgets_owner ON reddit_api_budgets(owner_id);
