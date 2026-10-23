'use client';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['ideas'];

export default function ProjectIdeas({ opportunities }: Props) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <span className="eyebrow">OPEN-SOURCE PROJECT CANDIDATES</span>
        <h2>Open-source project ideas</h2>
        <p>These themes are prompts for open-source collaboration, not community endorsements or verified demand.</p>
      </div>
      {opportunities.length ? (
        <div className="list">
          {opportunities.map((o) => (
            <article className="list-item" key={o.id}>
              <div className="score">{Math.round(o.score)} / 100</div>
              <h3>{o.title}</h3>
              <p>{o.problem}</p>
              <small>{o.evidence_refs.length} supporting references</small>
            </article>
          ))}
        </div>
      ) : (
        <p className="empty">Review community themes to generate ideas for useful open-source contributions.</p>
      )}
    </section>
  );
}
