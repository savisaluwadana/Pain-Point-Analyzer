ALTER TABLE research_jobs ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now();
CREATE INDEX IF NOT EXISTS idx_jobs_retry ON research_jobs(status, next_attempt_at, created_at) WHERE status IN ('queued','running');
