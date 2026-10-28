export type Evidence = {
  id: string; source_kind: 'manual' | 'demo'; source_id: string; source_url: string;
  title: string; body: string; created_at: string;
};
export type Cluster = { category: string; independent_stories: number; evidence_count: number;
  severity_score: number; sufficient_evidence: boolean; evidence_ids: string[]; source_families: number };
export type Run = { id: string; topic: string; report: { engine: string; evidence_scanned: number;
  clusters: Cluster[]; caveat: string; unclassified_items: number; independent_stories: number }; created_at: string };
export type Decision = 'unreviewed' | 'investigating' | 'needs_validation' | 'ready_to_build' | 'not_pursuing';
export type Opportunity = { id: string; run_id: string; title: string; score: number;
  independent_stories: number; evidence_refs: string[]; decision: Decision;
  existing_solutions: string; validation_notes: string; created_at: string };
export type Workspace = { evidence: Evidence[]; runs: Run[]; opportunities: Opportunity[] };
