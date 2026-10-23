'use client';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['assisted'];

export default function AssistedResearch({ hostJobs, hostRuns, hostWorkspaces, load }: Props) {
  return (
    <div className="two-col">
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">OPTIONAL ASSISTED RESEARCH</span>
          <h2>Research jobs</h2>
          <p>
            Codex or Claude can browse approved sources, supply grounded evidence and ask the Rust API to persist
            research progress.
          </p>
        </div>
        <button className="secondary" onClick={() => void load()}>
          Refresh agent state
        </button>
        {hostJobs.length ? (
          <div className="list">
            {hostJobs.map((j) => (
              <article className="list-item" key={j.id}>
                <span className="pill">{j.status}</span>
                <h3>{j.topic}</h3>
                <small>Research pass: {j.pass || 0}</small>
              </article>
            ))}
          </div>
        ) : (
          <p className="empty">No host-driven jobs yet. Connect an MCP host to start one.</p>
        )}
      </section>
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">SEMANTIC RESEARCH</span>
          <h2>LLM-assisted reports</h2>
          <p>
            Host-provided analysis must reference evidence IDs; the server does not invent or generate source material.
          </p>
        </div>
        {hostRuns.length ? (
          <div className="list">
            {hostRuns.map((r) => (
              <article className="list-item" key={r.id}>
                <span className="pill">{r.status}</span>
                <h3>{r.name || r.topic}</h3>
                <p>
                  {Object.keys(r.annotations || {}).length} annotations · {(r.clusters || []).length} clusters ·{' '}
                  {(r.opportunities || []).length} candidates
                </p>
              </article>
            ))}
          </div>
        ) : (
          <p className="empty">No host-assisted research reviews yet.</p>
        )}
      </section>
      <section className="panel span-all">
        <div className="panel-heading">
          <span className="eyebrow">FROM RESEARCH TO CONTRIBUTION</span>
          <h2>Opportunity workspaces</h2>
          <p>Evidence-linked ideas, community validation, and planning for useful open-source contributions.</p>
        </div>
        {hostWorkspaces.length ? (
          <div className="cluster-grid">
            {hostWorkspaces.map((w) => (
              <div className="cluster" key={w.id}>
                <span className="pill">{w.stage}</span>
                <h3>{w.opportunity?.title || 'Untitled opportunity'}</h3>
                <p>{w.opportunity?.problem || ''}</p>
                <small>{(w.experiments || []).length} validation experiments</small>
              </div>
            ))}
          </div>
        ) : (
          <p className="empty">Create opportunity workspaces through the MCP research workflow.</p>
        )}
      </section>
    </div>
  );
}
