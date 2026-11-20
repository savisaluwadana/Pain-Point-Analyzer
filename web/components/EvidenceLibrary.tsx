'use client';
import { api, safeSourceUrl } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';
import ProjectScope from './ProjectScope';

type Props = Workspace['context'] & Workspace['evidence'] & Workspace['scope'];

export default function EvidenceLibrary({
  user,
  busy,
  act,
  project_id,
  projects,
  selected,
  setSelected,
  items,
  manualTitle,
  setManualTitle,
  manualBody,
  setManualBody,
  manualUrl,
  setManualUrl,
}: Props) {
  return (
    <>
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">MANUAL EVIDENCE</span>
          <h2>Add research evidence</h2>
          <p>Add an independently collected source where you have permission to retain the material.</p>
        </div>
        <ProjectScope {...{ projects, selected, setSelected, busy }} />
        <p className="disclaimer">
          New evidence is saved to the selected project, or unassigned when all projects are shown.
        </p>
        <label htmlFor="manualTitle">Evidence title</label>
        <input
          id="manualTitle"
          maxLength={500}
          value={manualTitle}
          onChange={(e) => setManualTitle(e.target.value)}
          placeholder="Community request or recurring challenge"
        />
        <label htmlFor="manualUrl">Source URL (optional)</label>
        <input
          id="manualUrl"
          type="url"
          maxLength={2048}
          value={manualUrl}
          onChange={(e) => setManualUrl(e.target.value)}
          placeholder="https://example.com/source"
        />
        <label htmlFor="manualBody">What did the source say?</label>
        <textarea
          id="manualBody"
          maxLength={10000}
          rows={4}
          value={manualBody}
          onChange={(e) => setManualBody(e.target.value)}
          placeholder="Summarize relevant evidence without unnecessary personal details"
        />
        <button
          className="primary"
          disabled={
            !user ||
            busy ||
            !manualTitle.trim() ||
            !manualBody.trim() ||
            (!!manualUrl.trim() && !safeSourceUrl(manualUrl.trim()))
          }
          onClick={() =>
            void act(async () => {
              await api('/evidence', {
                method: 'POST',
                body: JSON.stringify({
                  source_kind: 'manual',
                  source_id: crypto.randomUUID(),
                  source_url: manualUrl.trim(),
                  title: manualTitle.trim(),
                  body: manualBody.trim(),
                  project_id,
                }),
              });
              setManualTitle('');
              setManualBody('');
              setManualUrl('');
            }, 'Evidence saved')
          }
        >
          Save evidence →
        </button>
        {!!manualUrl.trim() && !safeSourceUrl(manualUrl.trim()) && (
          <p role="alert">Use a valid HTTP or HTTPS source URL.</p>
        )}
      </section>
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">SOURCE LIBRARY</span>
          <h2>Collected evidence</h2>
          <p>
            {items.length} recent records in {selected ? 'this project' : 'all projects'} (up to 200).
          </p>
        </div>
        {items.length ? (
          <div className="list">
            {items.map((e) => (
              <article key={e.id} className="list-item">
                <span className="pill">{e.source_kind}</span>
                <h3>{e.title || e.source_id}</h3>
                <p>{e.body.slice(0, 260)}</p>
                {e.source_kind === 'stackoverflow' && (
                  <p className="disclaimer">
                    Stack Overflow contribution
                    {e.metadata?.details?.author ? ' by ' + e.metadata.details.author : ''} ·{' '}
                    <a href="https://stackoverflow.com/help/licensing" target="_blank" rel="noopener noreferrer">
                      CC BY-SA (version may vary)
                    </a>
                    . Review the question and its license at the original link.
                  </p>
                )}
                <div className="evidence-actions">
                  {safeSourceUrl(e.source_url) && (
                    <a href={safeSourceUrl(e.source_url)} target="_blank" rel="noopener noreferrer">
                      View original source ↗
                    </a>
                  )}
                  <button
                    className="tiny"
                    disabled={busy}
                    onClick={() => {
                      if (window.confirm('Delete this evidence item? Saved analysis reports are historical snapshots.'))
                        void act(() => api('/evidence/' + e.id, { method: 'DELETE' }), 'Evidence deleted');
                    }}
                  >
                    Delete evidence
                  </button>
                </div>
              </article>
            ))}
          </div>
        ) : (
          <p className="empty">No evidence collected.</p>
        )}
      </section>
    </>
  );
}
