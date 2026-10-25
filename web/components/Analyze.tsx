'use client';
import { useState } from 'react';
import { api } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';
import Projects from './Projects';
type Props = Workspace['context'] & Workspace['scope'] & Workspace['analysis'];

export default function Analyze({
  user,
  busy,
  act,
  projects,
  selected,
  setSelected,
  project_id,
  subreddit,
  setSubreddit,
  threadId,
  setThreadId,
  topic,
  setTopic,
  items,
  latest,
  projectEditor,
}: Props) {
  const [analysisLimit, setAnalysisLimit] = useState(1000);
  return (
    <div className="two-col">
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">01 / DISCOVER</span>
          <h2>Collect evidence</h2>
          <p>
            With approved Reddit OAuth, collect a small read-only sample of community discussions for local research.
          </p>
        </div>
        {!user?.reddit_connected && (
          <p className="disclaimer">
            Use Community Sources for the archive and other connectors. The optional Reddit API controls below require
            an approved, connected Reddit account.
          </p>
        )}
        <label htmlFor="project">Research project</label>
        <select id="project" value={selected} onChange={(e) => setSelected(e.target.value)}>
          <option value="">All evidence / no project</option>
          {projects.map((p) => (
            <option value={p.id} key={p.id}>
              {p.name}
            </option>
          ))}
        </select>
        <label htmlFor="subreddit">Subreddit</label>
        <div className="prefix-input">
          <span>r/</span>
          <input
            id="subreddit"
            value={subreddit}
            onChange={(e) => setSubreddit(e.target.value)}
            placeholder="opensource"
          />
        </div>
        <button
          className="primary"
          disabled={!user?.reddit_connected || busy}
          onClick={() =>
            void act(
              () =>
                api('/research/scan', {
                  method: 'POST',
                  body: JSON.stringify({ subreddit, project_id, limit: 25 }),
                }),
              'Authorized Reddit sample collected',
            )
          }
        >
          {busy ? 'Processing…' : 'Collect Reddit sample →'}
        </button>
        <button
          className="secondary"
          disabled={!user?.reddit_connected || busy}
          onClick={() =>
            void act(
              () => api('/research/jobs', { method: 'POST', body: JSON.stringify({ subreddit, project_id }) }),
              'Research job queued',
            )
          }
        >
          Queue a background collection
        </button>
        <div className="subpanel">
          <span className="eyebrow">THREAD COMMENTS</span>
          <label htmlFor="thread">Reddit post ID</label>
          <input
            id="thread"
            placeholder="e.g. 1abcxyz"
            value={threadId}
            onChange={(e) => setThreadId(e.target.value)}
          />
          <button
            className="secondary"
            disabled={!user?.reddit_connected || busy || !threadId.trim()}
            onClick={() =>
              void act(
                () =>
                  api('/research/thread', {
                    method: 'POST',
                    body: JSON.stringify({ post_id: threadId, project_id, limit: 50 }),
                  }),
                'Thread comments collected',
              )
            }
          >
            Collect authorized comments
          </button>
        </div>
      </section>
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">02 / ANALYZE</span>
          <h2>Discover recurring themes</h2>
          <p>
            Group recurring challenges and feature requests into evidence-linked themes. Review findings with
            communities before proposing a project.
          </p>
        </div>
        <label htmlFor="topic">Research question</label>
        <textarea id="topic" rows={3} value={topic} onChange={(e) => setTopic(e.target.value)} />
        <label htmlFor="analysisLimit">Analysis budget</label>
        <select id="analysisLimit" value={analysisLimit} onChange={(e) => setAnalysisLimit(Number(e.target.value))}>
          <option value={1000}>Up to 1,000 records</option>
          <option value={5000}>Up to 5,000 records</option>
          <option value={10000}>Up to 10,000 records</option>
        </select>
        <button
          className="primary"
          disabled={!user || busy || items.length === 0}
          onClick={() =>
            void act(
              () =>
                api('/research/analyze', {
                  method: 'POST',
                  body: JSON.stringify({ topic, project_id, limit: analysisLimit }),
                }),
              'Community research review saved',
            )
          }
        >
          Run evidence analysis →
        </button>
        <Projects {...projectEditor} />
      </section>
      <section className="panel span-all">
        <div className="panel-heading">
          <span className="eyebrow">LATEST ANALYSIS</span>
          <h2>{latest?.topic || 'No research report yet'}</h2>
        </div>
        {latest ? (
          <>
            <div className="summaryline">
              {latest.report.evidence_scanned} items scanned · {latest.report.independent_stories} observed source
              stories
            </div>
            {latest.report.coverage && (
              <p>
                {latest.report.coverage.selected_count} of {latest.report.coverage.candidate_count} candidate records
                selected.
                {latest.report.coverage.omitted_count > 0 && (
                  <strong>
                    {' '}
                    {latest.report.coverage.omitted_count} records omitted by the analysis limit. Narrow your scope
                    before drawing conclusions.
                  </strong>
                )}
                {!!latest.report.coverage.topic_excluded_count && (
                  <> {latest.report.coverage.topic_excluded_count} candidates excluded by the topic relevance rules.</>
                )}
              </p>
            )}
            {latest.report.unclassified_items !== undefined && (
              <p>{latest.report.unclassified_items} items did not match complaint rules.</p>
            )}
            {latest.report.score_version === 2 && (
              <p>
                Rule-based analysis · raw priority = 0.70 × ln(1 + independent stories) + 0.15 × ln(1 + source families)
                + 0.15 × complaint strength. Relative priority compares themes within this run; it is not severity or a
                probability. A single theme receives 50 because there is no comparison.
              </p>
            )}
            <div className="cluster-grid">
              {latest.report.clusters.length === 0 && (
                <p className="empty">
                  No complaint rules matched this sample. This does not prove that no problem exists. Try a different
                  scope or review the evidence manually.
                </p>
              )}
              {latest.report.clusters.map((c) => (
                <div className="cluster" key={c.category}>
                  <div className="cluster-top">
                    <b>{c.category}</b>
                    <span>
                      {latest.report.score_version === 2 ? 'Relative priority: ' : 'Legacy score: '}
                      {Math.round(c.severity_score)}/100
                    </span>
                  </div>
                  <div className="bar">
                    <div style={{ width: c.severity_score + '%' }} />
                  </div>
                  <small>
                    {c.independent_stories} independent stories · {c.evidence_count} signals
                  </small>
                  {c.raw_priority !== undefined && (
                    <p>
                      Raw priority: {c.raw_priority.toFixed(3)} · {c.source_families} source families
                    </p>
                  )}
                  {c.sufficient_evidence === false && (
                    <p>Insufficient evidence: fewer than three independent stories.</p>
                  )}
                  {c.supporting_evidence?.map((e) => (
                    <div key={e.evidence_id}>
                      {e.available ? (
                        <blockquote>
                          <p>{e.excerpt}</p>
                          {e.source_url && /^https?:\/\//i.test(e.source_url) && (
                            <a href={e.source_url} target="_blank" rel="noopener noreferrer">
                              Read original source
                            </a>
                          )}
                        </blockquote>
                      ) : (
                        <p>Supporting evidence expired, changed or was deleted.</p>
                      )}
                    </div>
                  ))}
                </div>
              ))}
            </div>
            <p className="disclaimer">{latest.report.caveat}</p>
          </>
        ) : (
          <p className="empty">
            Collect evidence and run your first analysis to see source-grounded community research.
          </p>
        )}
      </section>
    </div>
  );
}
