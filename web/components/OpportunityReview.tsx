'use client';
import { useState } from 'react';
import { api } from '../lib/api';
import type { Opportunity, OpportunityBrief, OpportunityDecision } from '../lib/types';

type Props = {
  opportunity: Opportunity;
  busy: boolean;
  act: (action: () => Promise<unknown>, message: string) => Promise<void>;
};

const decisions: { value: OpportunityDecision; label: string }[] = [
  { value: 'unreviewed', label: 'Not reviewed' },
  { value: 'investigating', label: 'Investigating' },
  { value: 'needs_validation', label: 'Needs independent validation' },
  { value: 'ready_to_build', label: 'Ready to build (manual decision)' },
  { value: 'not_pursuing', label: 'Not pursuing' },
];

function safeSourceUrl(value?: string) {
  if (!value) return undefined;
  try {
    const url = new URL(value);
    return url.protocol === 'https:' || url.protocol === 'http:' ? url.href : undefined;
  } catch {
    return undefined;
  }
}

export default function OpportunityReview({ opportunity, busy, act }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [loading, setLoading] = useState(false);
  const [brief, setBrief] = useState<OpportunityBrief | null>(null);
  const [error, setError] = useState('');
  const [decision, setDecision] = useState<OpportunityDecision>(opportunity.decision || 'unreviewed');
  const [notes, setNotes] = useState(opportunity.validation_notes || '');
  const [solutions, setSolutions] = useState(opportunity.existing_solutions || '');
  const id = opportunity.id;

  async function toggle() {
    if (expanded) {
      setExpanded(false);
      return;
    }
    setExpanded(true);
    if (brief) return;
    setLoading(true);
    setError('');
    try {
      setBrief(await api<OpportunityBrief>('/opportunities/' + encodeURIComponent(id) + '/brief'));
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Could not load supporting evidence');
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="subpanel">
      <button className="secondary" type="button" aria-expanded={expanded} onClick={() => void toggle()}>
        {expanded ? 'Hide opportunity review' : 'Review evidence and decision'}
      </button>
      {expanded && (
        <div>
          <h4>Evidence-backed review</h4>
          {loading && <p role="status">Loading evidence…</p>}
          {error && (
            <p role="alert">
              {error}{' '}
              <button
                className="tiny"
                type="button"
                onClick={() => {
                  setExpanded(false);
                  setBrief(null);
                }}
              >
                Close and retry
              </button>
            </p>
          )}
          {brief && (
            <>
              <p>
                {brief.cluster?.independent_stories ?? 'Unknown'} independent discussions ·{' '}
                {brief.cluster?.source_families ?? 'Unknown'} source families · {brief.referenced_count} historical
                references
              </p>
              {brief.coverage && (
                <p>
                  Scope: {brief.coverage.selected_count} selected of {brief.coverage.candidate_count} candidates
                  {brief.coverage.omitted_count > 0 && ' (truncated)'}.
                </p>
              )}
              {brief.cluster?.sufficient_evidence === false && (
                <p>Fewer than three independent supporting stories. Investigate before drawing conclusions.</p>
              )}
              {brief.cluster?.supporting_evidence?.length ? (
                <div className="list">
                  {brief.cluster.supporting_evidence.map((e) => (
                    <div className="list-item" key={e.evidence_id}>
                      {e.available ? (
                        <>
                          <p>{e.excerpt}</p>
                          {safeSourceUrl(e.source_url) && (
                            <a target="_blank" rel="noopener noreferrer" href={safeSourceUrl(e.source_url)}>
                              Read original source
                            </a>
                          )}
                        </>
                      ) : (
                        <p>Supporting evidence is missing, changed, or expired.</p>
                      )}
                    </div>
                  ))}
                </div>
              ) : (
                <p>
                  No preserved representative excerpts for this run. Review the evidence library and original sources.
                </p>
              )}
              <p className="disclaimer">{brief.caveat}</p>
            </>
          )}
          <label htmlFor={'solutions-' + id}>Existing solutions and alternatives</label>
          <textarea
            id={'solutions-' + id}
            maxLength={2000}
            rows={3}
            value={solutions}
            onChange={(e) => setSolutions(e.target.value)}
            placeholder="Record links or your own comparison of existing open-source tools."
          />
          <label htmlFor={'notes-' + id}>Validation notes</label>
          <textarea
            id={'notes-' + id}
            maxLength={4000}
            rows={4}
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            placeholder="What still needs testing? Record your own interview findings and contradictory evidence."
          />
          <p className="disclaimer">
            These are your own notes. Do not copy third-party posts, personal data or copyrighted excerpts. Choosing a
            decision is not proof of market demand.
          </p>
          <label htmlFor={'decision-' + id}>Your next decision</label>
          <select
            id={'decision-' + id}
            value={decision}
            onChange={(e) => setDecision(e.target.value as OpportunityDecision)}
          >
            {decisions.map((choice) => (
              <option key={choice.value} value={choice.value}>
                {choice.label}
              </option>
            ))}
          </select>
          <button
            className="primary"
            disabled={busy || (decision === 'ready_to_build' && notes.trim().length < 20)}
            type="button"
            onClick={() =>
              void act(
                () =>
                  api('/opportunities/' + encodeURIComponent(id) + '/review', {
                    method: 'PUT',
                    body: JSON.stringify({ decision, validation_notes: notes, existing_solutions: solutions }),
                  }),
                'Opportunity review saved',
              )
            }
          >
            Save manual review
          </button>
        </div>
      )}
    </div>
  );
}
