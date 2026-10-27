export type User = { id: string; reddit_username: string; reddit_connected: boolean; local_account?: boolean };
export type Project = { id: string; name: string; description: string; created_at: string };
export type Evidence = {
  id: string;
  source_kind: string;
  source_id: string;
  source_url: string;
  title: string;
  body: string;
  metadata?: { details?: { author?: string; license?: string; license_url?: string } };
  collected_at: string;
};
export type Cluster = {
  category: string;
  independent_stories: number;
  evidence_count: number;
  severity_score: number;
  examples: string[];
  raw_priority?: number;
  sufficient_evidence?: boolean;
  source_families?: number;
  mean_complaint_strength?: number;
  supporting_evidence?: { evidence_id: string; available: boolean; excerpt?: string; source_url?: string }[];
};
export type Report = {
  evidence_scanned: number;
  independent_stories: number;
  clusters: Cluster[];
  caveat: string;
  engine?: string;
  score_version?: number;
  unclassified_items?: number;
  coverage?: {
    candidate_count: number;
    selected_count: number;
    omitted_count: number;
    topic_excluded_count?: number;
    limit: number;
  };
};
export type Run = { id: string; project_id: string | null; topic: string; report: Report; created_at: string };
export type Job = {
  id: string;
  subreddit: string;
  status: string;
  attempts: number;
  last_error: string | null;
  created_at: string;
};
export type HostRun = {
  id: string;
  topic: string;
  name: string;
  status: string;
  clusters: unknown[];
  opportunities: unknown[];
  annotations: Record<string, unknown>;
};
export type HostWorkspace = {
  id: string;
  stage: string;
  opportunity: { title: string; problem: string };
  experiments: unknown[];
};
export type HostJob = { id: string; topic: string; status: string; pass: number };
export type ApiKey = { id: string; label: string; created_at: string; last_used_at: string | null };
export type Opportunity = {
  id: string;
  run_id?: string | null;
  score_version?: number | null;
  project_id: string | null;
  title: string;
  problem: string;
  score: number;
  evidence_refs: string[];
  decision?: OpportunityDecision;
  validation_notes?: string;
  existing_solutions?: string;
  reviewed_at?: string | null;
  created_at: string;
};
export type OpportunityDecision =
  'unreviewed' | 'investigating' | 'needs_validation' | 'ready_to_build' | 'not_pursuing';
export type OpportunityBrief = {
  engine?: string;
  title: string;
  referenced_count: number;
  coverage?: { candidate_count: number; selected_count: number; omitted_count: number };
  cluster: {
    independent_stories?: number;
    source_families?: number;
    sufficient_evidence?: boolean;
    supporting_evidence?: { evidence_id: string; available: boolean; excerpt?: string; source_url?: string }[];
  } | null;
  caveat: string;
};
export type SourceOption = {
  id: string;
  name: string;
  mode: 'keyword' | 'video' | 'forum' | 'tag' | 'subreddit';
  ready: boolean;
  note: string;
};
export type SourceCatalog = { sources: SourceOption[] };
export type Tab =
  'discover' | 'sources' | 'evidence' | 'opportunities' | 'jobs' | 'integrations' | 'agents' | 'settings';
