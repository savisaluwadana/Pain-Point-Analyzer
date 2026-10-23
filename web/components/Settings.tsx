'use client';
import { api } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['context'];

export default function Settings({ user, busy, act }: Props) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <span className="eyebrow">ACCOUNT PRIVACY</span>
        <h2>Your research data and Reddit access</h2>
        <p>These controls operate on your account only. Research and account deletion cannot be undone.</p>
      </div>
      <div className="subpanel">
        <h3>Export your research</h3>
        <p>
          Download your projects, evidence, saved analyses and opportunity workspaces. Source-content licensing still
          applies to exported data. API credentials and Reddit post/comment text are excluded.
        </p>
        {user ? (
          <a className="connect" href="/api/privacy/export" download="pain-intelligence-export.json">
            Download research JSON ↓
          </a>
        ) : (
          <p>Sign in to download your research.</p>
        )}
      </div>
      <div className="subpanel">
        <h3>Disconnect Reddit</h3>
        <p>
          Revoke this app&apos;s Reddit OAuth token, stop pending Reddit collections and delete stored Reddit posts and
          comments. Other projects, research summaries and MCP keys remain. To erase those too, delete the account.
        </p>
        <button
          className="secondary"
          disabled={!user || !user.reddit_connected || busy}
          onClick={() => {
            if (window.confirm('Revoke Reddit access and delete stored Reddit post/comment evidence?'))
              void act(
                () => api('/auth/reddit/disconnect', { method: 'POST' }),
                'Reddit disconnected and source evidence removed',
              );
          }}
        >
          Disconnect and revoke Reddit access
        </button>
      </div>
      <div className="subpanel">
        <h3>Delete account and all stored data</h3>
        <p>
          Delete this account, sessions, projects, evidence, research reports, opportunity workspaces and API keys. The
          service also attempts to revoke Reddit&apos;s token before deletion. If that fails, revoke the app manually in
          Reddit account settings.
        </p>
        <button
          className="secondary"
          disabled={!user || busy}
          onClick={() => {
            if (
              window.confirm('Permanently delete your account and ALL its stored research data? This cannot be undone.')
            )
              void act(() => api('/privacy/delete-data', { method: 'POST' }), 'Your stored account data was deleted');
          }}
        >
          Delete my account and data
        </button>
      </div>
    </section>
  );
}
