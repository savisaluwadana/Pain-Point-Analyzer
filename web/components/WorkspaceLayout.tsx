'use client';
import type { ReactNode } from 'react';
import type { Tab, User } from '../lib/types';
import type { Workspace } from '../lib/use-workspace';

type Props = {
  tab: Tab;
  setTab: (tab: Tab) => void;
  user: User | null;
  notice: string;
  loadError: string;
  loading: boolean;
  busy: boolean;
  onRefresh: () => Promise<void>;
  counts: Workspace['counts'];
  onSignOut: () => Promise<void>;
  login: ReactNode;
  children: ReactNode;
};

export default function WorkspaceLayout({
  tab,
  setTab,
  user,
  notice,
  loadError,
  loading,
  busy,
  onRefresh,
  counts,
  onSignOut,
  login,
  children,
}: Props) {
  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-symbol">◈</span>
          <span>
            Open Source<span className="brand-os"> Discovery Lab</span>
          </span>
        </div>
        <p className="sidebar-caption">From community conversations to useful open-source contributions.</p>
        <div className="side-section">WORKSPACE</div>
        <nav aria-label="Research navigation">
          {(
            [
              ['sources', '⌁', 'Community Sources'],
              ['discover', '◉', 'Analyze & Discover'],
              ['evidence', '≡', 'Evidence Library'],
              ['opportunities', '◇', 'Project Ideas'],
              ['jobs', '↻', 'Research Jobs'],
              ['agents', '♧', 'Assisted Research'],
              ['integrations', '⌘', 'MCP Integrations'],
              ['settings', '⚙', 'Privacy & Settings'],
            ] as const
          ).map(([key, icon, label]) => (
            <button
              key={key}
              onClick={() => setTab(key)}
              aria-current={tab === key ? 'page' : undefined}
              className={'side-link ' + (tab === key ? 'active' : '')}
            >
              <span aria-hidden="true">{icon}</span>
              {label}
            </button>
          ))}
        </nav>
        <div className="sidebar-bottom">
          <span className="status-dot" /> Rust + PostgreSQL
          <p>Public APIs and approved sources only. Research themes require community validation.</p>
        </div>
      </aside>
      <main className="main">
        <header className="topbar">
          <span>
            OPEN-SOURCE RESEARCH WORKSPACE <span className="sep">/</span> {tab.toUpperCase()}
          </span>
          <div className="identity">
            {user ? (
              <>
                <span className="avatar">{user.reddit_username.slice(0, 1).toUpperCase()}</span>
                <b>{user.local_account ? 'Local workspace' : 'u/' + user.reddit_username}</b>
                {!user.reddit_connected && !user.local_account && (
                  <a className="connect" href="/api/auth/reddit/start">
                    Reconnect Reddit
                  </a>
                )}
                <button disabled={busy} onClick={() => void onSignOut()}>
                  Sign out
                </button>
              </>
            ) : (
              <a className="connect" href="/api/auth/reddit/start">
                Connect Reddit (optional) →
              </a>
            )}
          </div>
        </header>
        <section className="intro">
          <div>
            <div className="eyebrow">COMMUNITY-FIRST DISCOVERY</div>
            <h1>
              Discover needs. <em>Build in the open.</em>
            </h1>
            <p>
              Explore permitted community discussions, find recurring needs, and shape ideas for useful open-source
              projects. Currently intended for individual, non-commercial research.
            </p>
          </div>
          <span className="edition">OPEN-SOURCE PROJECT DISCOVERY</span>
        </section>
        {notice && (
          <div className="notice" role="status">
            {notice}
          </div>
        )}
        {loadError && (
          <div className="notice" role="alert">
            {loadError}{' '}
            <button className="tiny" disabled={busy} onClick={() => void onRefresh()}>
              Retry loading
            </button>
          </div>
        )}
        {loading && (
          <p role="status" className="loading-state">
            Loading workspace…
          </p>
        )}
        {login}
        {user && !user.reddit_connected && !user.local_account && (
          <div className="notice">Reddit is disconnected. Your existing non-Reddit research remains available.</div>
        )}
        {user && (
          <section className="stats" aria-label="Research summary">
            {[
              ['Evidence displayed', counts.items.toString(), 'Latest 200 records loaded'],
              ['Research reviews', counts.runs.toString(), 'Latest reports loaded'],
              ['Open-source ideas', counts.opportunities.toString(), 'Require community validation'],
              ['Reddit jobs', counts.jobs.toString(), 'Queue and history'],
            ].map(([label, value, caption]) => (
              <div className="stat" key={label}>
                <span>{label}</span>
                <strong>{value}</strong>
                <small>{caption}</small>
              </div>
            ))}
          </section>
        )}

        {user && <div aria-busy={loading}>{children}</div>}
        <footer>
          OPEN SOURCE DISCOVERY LAB <span>© 2026 · Local, currently non-commercial community research</span>
        </footer>
      </main>
    </div>
  );
}
