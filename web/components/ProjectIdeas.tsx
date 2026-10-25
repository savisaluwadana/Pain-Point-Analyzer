'use client';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['ideas'];

export default function ProjectIdeas({ opportunities }: Props) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <span className="eyebrow">OPEN-SOURCE PROJECT CANDIDATES</span>
        <h2>Open-source project ideas</h2>
        <p>
          Newest research first. These themes are investigation prompts, not verified project demand. Compare relative
          priorities only within the same research run.
        </p>
      </div>
      {opportunities.length ? (
        <div className="list">
          {opportunities.map((o) => (
            <article className="list-item" key={o.id}>
              <div className="score">
                {o.score_version === 2 ? 'Relative priority' : 'Legacy score'}: {Math.round(o.score)} / 100
              </div>
              <h3>{o.title}</h3>
              <p>{o.problem}</p>
              <small>
                {o.evidence_refs.length} supporting references
                {o.run_id ? ` · Research run ${o.run_id.slice(0, 8)}` : ''}
              </small>
            </article>
          ))}
        </div>
      ) : (
        <p className="empty">Review community themes to generate ideas for useful open-source contributions.</p>
      )}
    </section>
  );
}
