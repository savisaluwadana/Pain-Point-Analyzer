CREATE TABLE IF NOT EXISTS evidence_projects (
 owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 evidence_id UUID NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
 project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
 PRIMARY KEY(owner_id,evidence_id,project_id)
);
CREATE INDEX IF NOT EXISTS idx_evidence_projects_project ON evidence_projects(owner_id,project_id,evidence_id);
INSERT INTO evidence_projects(owner_id,evidence_id,project_id)
 SELECT owner_id,id,project_id FROM evidence WHERE project_id IS NOT NULL ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS evidence_batches (
 owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 evidence_id UUID NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
 batch_id TEXT NOT NULL CHECK(length(batch_id) BETWEEN 1 AND 120),
 PRIMARY KEY(owner_id,evidence_id,batch_id)
);
CREATE INDEX IF NOT EXISTS idx_evidence_batches_batch ON evidence_batches(owner_id,batch_id,evidence_id);
INSERT INTO evidence_batches(owner_id,evidence_id,batch_id)
 SELECT owner_id,id,metadata->>'batch_id' FROM evidence WHERE length(metadata->>'batch_id') BETWEEN 1 AND 120 ON CONFLICT DO NOTHING;
