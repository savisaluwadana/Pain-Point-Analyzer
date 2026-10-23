'use client';
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
          disabled={!user || busy}
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
          disabled={!user || busy}
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
            disabled={!user || busy || !threadId.trim()}
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
        <button
          className="primary"
          disabled={!user || busy || items.length === 0}
          onClick={() =>
            void act(
              () => api('/research/analyze', { method: 'POST', body: JSON.stringify({ topic, project_id }) }),
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
            <div className="cluster-grid">
              {latest.report.clusters.map((c) => (
                <div className="cluster" key={c.category}>
                  <div className="cluster-top">
                    <b>{c.category}</b>
                    <span>{Math.round(c.severity_score)}/100</span>
                  </div>
                  <div className="bar">
                    <div style={{ width: c.severity_score + '%' }} />
                  </div>
                  <small>
                    {c.independent_stories} independent stories · {c.evidence_count} signals
                  </small>
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
