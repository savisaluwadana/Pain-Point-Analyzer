-- Human review is separate from inferred opportunity score.
-- Notes are user-authored research, not evidence quotes or AI assertions.
ALTER TABLE opportunities
  ADD COLUMN decision TEXT NOT NULL DEFAULT 'unreviewed'
    CHECK (decision IN ('unreviewed','investigating','needs_validation','ready_to_build','not_pursuing')),
  ADD COLUMN validation_notes TEXT NOT NULL DEFAULT ''
    CHECK (char_length(validation_notes) <= 4000),
  ADD COLUMN existing_solutions TEXT NOT NULL DEFAULT ''
    CHECK (char_length(existing_solutions) <= 2000),
  ADD COLUMN reviewed_at TIMESTAMPTZ;
