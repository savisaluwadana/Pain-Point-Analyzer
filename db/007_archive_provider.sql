-- Shared across API replicas; reserves one request at a time and persists backoff.
CREATE TABLE archive_provider_budget (
 provider TEXT PRIMARY KEY,
 next_request_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
