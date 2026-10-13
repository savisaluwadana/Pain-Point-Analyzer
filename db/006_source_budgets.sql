-- Limit connector calls per source per local account. This budget supplements provider rate limits.
CREATE TABLE IF NOT EXISTS source_request_budgets (
  owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  source TEXT NOT NULL,
  window_started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  request_count INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(owner_id,source)
);
