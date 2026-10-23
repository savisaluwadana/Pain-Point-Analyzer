'use client';
import { api } from '../lib/api';
import type { Workspace } from '../lib/use-workspace';

type Props = Workspace['context'] & Workspace['integrations'];

export default function McpIntegrations({ user, busy, act, keys, keyLabel, setKeyLabel, newKey, setNewKey }: Props) {
  return (
    <section className="panel">
      <div className="panel-heading">
        <span className="eyebrow">NATIVE RUST MCP / HOST INTEGRATION</span>
        <h2>Connect Claude, Codex or another MCP host</h2>
        <p>
          Create a revocable, per-account key for the local Rust MCP server. Your host controls model reasoning and
          optional browsing; Reddit archive collection runs in the backend; source text remains untrusted evidence.
        </p>
      </div>
      <label htmlFor="mcpKeyLabel">Key description</label>
      <input
        id="mcpKeyLabel"
        value={keyLabel}
        onChange={(e) => setKeyLabel(e.target.value)}
        placeholder="My desktop MCP client"
      />
      <button
        className="primary"
        disabled={!user || busy || !keyLabel.trim()}
        onClick={() =>
          void act(async () => {
            const result = await api<{ api_key: string }>('/mcp/keys', {
              method: 'POST',
              body: JSON.stringify({ label: keyLabel }),
            });
            setNewKey(result.api_key);
          }, 'API key created. Copy and store it securely now.')
        }
      >
        Generate MCP API key
      </button>
      {newKey && (
        <div className="subpanel">
          <span className="eyebrow">ONE-TIME SECRET — COPY NOW</span>
          <p>Copy this key into your MCP host configuration. It will not be shown again.</p>
          <div className="row">
            <input aria-label="New API key" type="text" value={newKey} readOnly />
            <button className="tiny" onClick={() => void navigator.clipboard.writeText(newKey)}>
              Copy
            </button>
          </div>
        </div>
      )}
      <div className="subpanel">
        <span className="eyebrow">ACTIVE API KEYS</span>
        {keys.map((k) => (
          <div className="list-item job" key={k.id}>
            <b>{k.label}</b>
            <small>{new Date(k.created_at).toLocaleDateString()}</small>
            <button
              className="tiny"
              disabled={busy}
              onClick={() =>
                void act(async () => {
                  await api('/mcp/keys/' + k.id, { method: 'DELETE' });
                  setNewKey('');
                }, 'API key revoked')
              }
            >
              Revoke
            </button>
          </div>
        ))}
      </div>
      <div className="subpanel">
        <span className="eyebrow">RUN THE MCP BRIDGE</span>
        <p>
          Install Rust and run this command from the repository root with your secret set in{' '}
          <code>PAIN_MCP_API_KEY</code>. Keep API credentials out of committed files.
        </p>
        <pre className="command">cargo run --manifest-path api/Cargo.toml -- --mcp</pre>
        <p className="disclaimer">
          Requires a running web/API stack. The bridge exposes 58 tool schemas, including browser-free Reddit archive
          collection. Execution is authenticated and all workflow data stays in your PostgreSQL workspace.
        </p>
      </div>
    </section>
  );
}
