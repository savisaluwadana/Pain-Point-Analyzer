'use client';
import { api } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['context'] & Workspace['login'];

export default function LoginForm({ busy, act, localToken, setLocalToken }: Props) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <span className="eyebrow">LOCAL RESEARCH ACCESS</span>
        <h2>Sign in without Reddit</h2>
        <p>
          If LOCAL_LOGIN_TOKEN is configured, enter your locally generated token. No Reddit API approval is needed for
          other permitted sources.
        </p>
      </div>
      <label htmlFor="localToken">Local access token</label>
      <input
        id="localToken"
        type="password"
        autoComplete="off"
        value={localToken}
        onChange={(e) => setLocalToken(e.target.value)}
        placeholder="Paste the token from your local .env"
      />
      <button
        className="primary"
        disabled={busy || !localToken.trim()}
        onClick={() =>
          void act(async () => {
            await api('/auth/local', { method: 'POST', body: JSON.stringify({ token: localToken }) });
            setLocalToken('');
          }, 'Local research workspace connected')
        }
      >
        Sign in locally →
      </button>
      <p className="disclaimer">Alternatively, connect an approved Reddit account using the header link.</p>
    </section>
  );
}
