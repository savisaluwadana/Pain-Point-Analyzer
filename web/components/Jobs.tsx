'use client';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['jobView'];

export default function Jobs({ jobs, load }: Props) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <span className="eyebrow">AUTOMATION</span>
        <h2>Background collection jobs</h2>
        <p>Approved API access and a valid Reddit session are required.</p>
      </div>
      <button className="secondary" onClick={() => void load()}>
        Refresh jobs
      </button>
      {jobs.length ? (
        <div className="list">
          {jobs.map((j) => (
            <article className="list-item job" key={j.id}>
              <b>r/{j.subreddit}</b>
              <span className="pill">{j.status}</span>
              <small>Attempts: {j.attempts}</small>
              {j.last_error && <p>{j.last_error}</p>}
            </article>
          ))}
        </div>
      ) : (
        <p className="empty">No Reddit collection jobs. Queue one from Community Discovery.</p>
      )}
    </section>
  );
}
