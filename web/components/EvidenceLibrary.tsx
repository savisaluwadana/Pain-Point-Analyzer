'use client';
import { api } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['context'] & Workspace['evidence'] & Workspace['evidenceScope'];

export default function EvidenceLibrary({
  user,
  busy,
  act,
  project_id,
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
        <label htmlFor="manualTitle">Evidence title</label>
        <input
          id="manualTitle"
          value={manualTitle}
          onChange={(e) => setManualTitle(e.target.value)}
          placeholder="Community request or recurring challenge"
        />
        <label htmlFor="manualUrl">Source URL (optional)</label>
        <input
          id="manualUrl"
          value={manualUrl}
          onChange={(e) => setManualUrl(e.target.value)}
          placeholder="https://example.com/source"
        />
        <label htmlFor="manualBody">What did the source say?</label>
        <textarea
          id="manualBody"
          rows={4}
          value={manualBody}
          onChange={(e) => setManualBody(e.target.value)}
          placeholder="Summarize relevant evidence without unnecessary personal details"
        />
        <button
          className="primary"
          disabled={!user || busy || !manualTitle.trim()}
          onClick={() =>
            void act(async () => {
              await api('/evidence', {
                method: 'POST',
                body: JSON.stringify({
                  source_kind: 'manual',
                  source_id: crypto.randomUUID(),
                  source_url: manualUrl,
                  title: manualTitle,
                  body: manualBody,
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
      </section>
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">SOURCE LIBRARY</span>
          <h2>Collected evidence</h2>
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
                {e.source_url && (
                  <a href={e.source_url} target="_blank" rel="noopener noreferrer">
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
