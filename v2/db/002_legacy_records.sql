CREATE TABLE IF NOT EXISTS legacy_records (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  collection_name TEXT NOT NULL,
  legacy_id TEXT NOT NULL,
  payload JSONB NOT NULL,
  imported_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (owner_id,collection_name,legacy_id)
);
CREATE INDEX IF NOT EXISTS idx_legacy_owner_collection ON legacy_records(owner_id,collection_name);
