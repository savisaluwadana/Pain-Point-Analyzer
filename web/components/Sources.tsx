'use client';
import { api } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['context'] & Workspace['scope'] & Workspace['sources'];

export default function Sources({
  user,
  busy,
  act,
  projects,
  selected,
  setSelected,
  project_id,
  catalog,
  chosenSource,
  source,
  setSource,
  sourceQuery,
  setSourceQuery,
  sourceTarget,
  setSourceTarget,
  sourceResult,
  setSourceResult,
  archiveRanking,
  setArchiveRanking,
  archiveDays,
  setArchiveDays,
  archivePost,
  setArchivePost,
}: Props) {
  return (
    <div className="two-col">
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">READ-ONLY COMMUNITY CONNECTORS</span>
          <h2>Collect source-linked evidence</h2>
          <p>
            Alpha preview: start with GitHub or try the{' '}
            <a
              href="https://github.com/savisaluwadana/Reddit-post-Analyzer/blob/main/docs/DEMO.md"
              target="_blank"
              rel="noopener noreferrer"
            >
              offline sample walkthrough ↗
            </a>
            . Sources have quotas and may refuse access. No Reddit API credentials required.
          </p>
        </div>
        <label htmlFor="sourceSelect">Source</label>
        <select
          id="sourceSelect"
          value={source}
          onChange={(e) => {
            setSource(e.target.value);
            setSourceResult('');
            if (e.target.value === 'medium') setSourceQuery('open-source');
            else if (e.target.value === 'youtube' || e.target.value === 'arctic-shift') setSourceQuery('');
            else setSourceQuery('feature request');
            setSourceTarget('');
          }}
        >
          {catalog.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
              {s.ready ? '' : ' (needs configuration)'}
            </option>
          ))}
        </select>
        {chosenSource && <p className="disclaimer">{chosenSource.note}</p>}
        <label htmlFor="sourceQuery">
          {chosenSource?.mode === 'subreddit'
            ? 'Optional phrase filter within the sample'
            : chosenSource?.mode === 'tag'
              ? 'Medium topic tag'
              : chosenSource?.mode === 'video'
                ? 'Optional comment search filter'
                : 'Search topic or phrase'}
        </label>
        <input
          id="sourceQuery"
          value={sourceQuery}
          onChange={(e) => setSourceQuery(e.target.value)}
          placeholder={chosenSource?.mode === 'tag' ? 'open-source' : 'missing feature'}
        />
        {(chosenSource?.mode === 'video' || chosenSource?.mode === 'forum' || chosenSource?.mode === 'subreddit') && (
          <>
            <label htmlFor="sourceTarget">
              {chosenSource.mode === 'subreddit'
                ? 'Subreddit (without r/)'
                : chosenSource.mode === 'video'
                  ? 'YouTube video ID'
                  : 'Allowlisted Discourse hostname'}
            </label>
            <input
              id="sourceTarget"
              value={sourceTarget}
              onChange={(e) => setSourceTarget(e.target.value)}
              placeholder={
                chosenSource.mode === 'subreddit'
                  ? 'freelance'
                  : chosenSource.mode === 'video'
                    ? 'dQw4w9WgXcQ'
                    : 'meta.discourse.org'
              }
            />
          </>
        )}
        {source === 'arctic-shift' && (
          <>
            <label htmlFor="archiveRanking">Selection within the archive sample</label>
            <select id="archiveRanking" value={archiveRanking} onChange={(e) => setArchiveRanking(e.target.value)}>
              <option value="top">Highest archived scores</option>
              <option value="mid">Middle of score ranking</option>
              <option value="new">Newest</option>
            </select>
            <label htmlFor="archiveDays">Look back (days)</label>
            <input
              id="archiveDays"
              type="number"
              min={1}
              max={365}
              value={archiveDays}
              onChange={(e) => setArchiveDays(Number(e.target.value))}
            />
            <label htmlFor="archivePost">Optional post ID for comments</label>
            <input
              id="archivePost"
              value={archivePost}
              onChange={(e) => setArchivePost(e.target.value)}
              placeholder="Leave blank to collect posts"
            />
            <p className="disclaimer">
              Selects up to 10 items from at most 100 recent archive records. Scores may be stale. Wait at least 10
              seconds between calls. Use no more than 10 communities per research run.
            </p>
          </>
        )}
        <label htmlFor="sourceProject">Research project</label>
        <select id="sourceProject" value={selected} onChange={(e) => setSelected(e.target.value)}>
          <option value="">Unassigned evidence</option>
          {projects.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
        <button
          className="primary"
          disabled={!user || busy || !chosenSource || !chosenSource.ready}
          onClick={() => {
            setSourceResult('');
            void act(async () => {
              const result = await api<{ saved: number; fetched?: number; eligible?: number; note?: string }>(
                '/sources/collect',
                {
                  method: 'POST',
                  body: JSON.stringify({
                    source,
                    query: sourceQuery,
                    target: sourceTarget,
                    project_id,
                    limit: source === 'arctic-shift' ? 10 : 15,
                    ...(source === 'arctic-shift'
                      ? {
                          days: archiveDays,
                          ranking: archiveRanking,
                          sample_size: 100,
                          post_id: archivePost.trim() || undefined,
                        }
                      : {}),
                  }),
                },
              );
              setSourceResult(
                result.saved +
                  ' new or updated evidence records saved' +
                  (result.fetched !== undefined
                    ? ' from ' + result.fetched + ' fetched / ' + result.eligible + ' eligible. ' + result.note
                    : ''),
              );
            }, 'Source collection completed');
          }}
        >
          {busy ? 'Collecting…' : 'Collect source evidence →'}
        </button>
        {sourceResult && (
          <p className="disclaimer" role="status">
            {sourceResult}
          </p>
        )}
      </section>
      <section className="panel">
        <div className="panel-heading">
          <span className="eyebrow">CONNECTOR STATUS</span>
          <h2>Supported sources</h2>
        </div>
        <div className="list">
          {catalog.map((s) => (
            <div key={s.id} className="list-item">
              <b>{s.name}</b>
              <span className="pill">{s.ready ? 'Available to try' : 'Credentials / allowlist needed'}</span>
              <p>{s.note}</p>
            </div>
          ))}
        </div>
        <p className="disclaimer">
          Content rights, API quotas, and provider policies still apply. Confirm original context before building an
          open-source project.
        </p>
      </section>
    </div>
  );
}
