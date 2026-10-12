# Pain Intelligence Lab — Rust / PostgreSQL / Next.js migration (v2)

This is a parallel **migration candidate**, not a drop-in replacement for the original Express/MongoDB/Vite application. The existing application remains in the repository and is the source of truth until parity tests, historical data import and user acceptance pass.

## Implemented

- Rust Axum API, SQLx migrations and per-Reddit-user PostgreSQL records
- Next.js React dashboard for projects, evidence, research summaries, opportunities and jobs
- Reddit OAuth authorization-code flow, random CSRF state cookie, server-only AES-alternative ChaCha20-Poly1305 encrypted credential storage, opaque hashed session cookies and token refresh
- Bounded authenticated Reddit API collection, rate-limit response handling, monthly top-post discovery and comment endpoint
- Heuristic clustering, independent story counts, persisted reports, background jobs claimed atomically with lease tokens
- Per-account complete data deletion, seven-day default expiry for new Reddit evidence, bounded retry backoff and an idempotent MongoDB archive/import utility
- Compose runtime and distinct CI for Rust / Next / Compose validation

## Setup

1. Obtain **approved** Reddit Data API application credentials. Register your app with the Reddit Developer Platform team. OAuth tokens are not permission for general scraping or commercial Reddit data usage. Obtain written commercial permission before using for a monetized product.
2. Copy `v2/.env.example` to `v2/.env`. Generate credentials with `openssl rand -hex 32`; set `TOKEN_ENCRYPTION_KEY_HEX`, a strong `POSTGRES_PASSWORD`, Reddit client ID/secret and unique User-Agent. Set `REDDIT_API_APPROVED=true` **only if approval exists**.
3. Register exact redirect URI `http://localhost:3000/api/auth/reddit/callback` with Reddit. Use HTTPS and `COOKIE_SECURE=true` outside localhost.
4. From `v2/` run `docker compose up --build -d --wait`. Open `http://localhost:3000`.
5. Click **Connect Reddit**, grant only identity/read scopes, create a project, collect a permitted subreddit sample, run analysis and inspect source-grounded candidates.
6. Run `docker compose logs -f api web` for startup diagnostics; `docker compose down` preserves PostgreSQL data.

### Local development

Start PostgreSQL and export `DATABASE_URL`, `REDDIT_REDIRECT_URI`, `WEB_URL`, `TOKEN_ENCRYPTION_KEY_HEX`, Reddit credentials and consent settings. Use `cd v2/api && cargo run`, and in another terminal `cd v2/web && npm install && npm run dev`. The Next.js development rewrite proxies /api requests to localhost:4000.

### API

| Method | Path | Purpose |
| --- | --- | --- |
| GET | /api/health | PostgreSQL readiness |
| GET | /api/auth/reddit/start | OAuth consent redirect |
| GET | /api/auth/reddit/callback | Authorization code exchange, encrypted token persistence |
| GET | /api/me | Current Reddit identity |
| POST | /api/auth/logout | Revoke local session |
| GET | /api/reddit/posts | Authenticated bounded top listing |
| GET | /api/reddit/comments | Authenticated comment listing |
| GET/POST | /api/projects | Research projects |
| GET/POST | /api/evidence | Evidence list / manual evidence ingest |
| POST | /api/research/scan | Authenticated Reddit ingest |
| POST | /api/research/analyze | Deterministic categorized findings |
| GET | /api/research/runs | Report history |
| GET/POST | /api/research/jobs | Queue / inspect collection jobs |
| GET | /api/opportunities | Evidence-linked research candidates |
| POST | /api/privacy/delete-data | Remove account and dependent records |

Every stateful endpoint other than OAuth start/callback and health requires the server-issued session cookie.

## MongoDB -> PostgreSQL preservation

**Do not delete the existing MongoDB volume.** Stop v1 writes, take and test a `mongodump` backup, and separately verify a PostgreSQL backup/restore.

1. Run v2, OAuth-connect the destination account so `users` contains the target identity.
2. In `v2/migrate`, run `npm install`, set `MONGO_URI`, `POSTGRES_DATABASE_URL` and `TARGET_REDDIT_USERNAME`.
3. Run `npm run import` for a read-only source count.
4. Review source collection retention/PII and ensure import is authorized, then run `npm run import -- --apply`.
5. Verify `legacy_records` counts against all old collections, and inspect projected projects/evidence. Every legacy record is preserved in owner-scoped JSONB with its collection and original ID; legacy Reddit entries are marked **legacy-reddit**, not newly OAuth-collected.
6. Compare representative historical runs, snapshots, clusters, quality assessments and MCP outputs against v1 before approving a cutover. Those features are **archived**, not all yet reconstructed as native v2 workflows.
7. Keep both backups and v1 available until reconciliation and rollback tests pass.

The archival table can contain personal information and old research data. Only import records you have rights to retain, protect backups and use the account deletion endpoint when appropriate.

## Boundaries / not yet parity-complete

- The former host-backed MCP bridge exposes 57 tools in v1. Those tool contracts, multi-source research orchestration, scraping intelligence, quality/consensus assessments and advanced Opportunity OS have **not yet been ported** to native Rust. Keep v1 available for those workflows.
- OAuth approval is an operator-configured flag, not approval supplied by Reddit. Reddit may deny new client credentials or change access.
- The typed scoring engine is *heuristic* (not LLM reasoning). Its scores rank investigation only and do not predict revenue or purchasing.
- Initial list endpoints are bounded, not fully cursor-paginated.
- Runs should be tested under concurrency and the complete stack should be smoke-tested before any production deployment.
- Multi-tenant SaaS plans, roles, billing and semantic model adapters are not implemented.
- **Do not merge** this branch as a replacement for v1 until remaining parity and retention/compliance work is finished.

## Privacy, security and policy

Access and refresh tokens are encrypted server-side using the key held in `TOKEN_ENCRYPTION_KEY_HEX`. Session tokens are random, hashed in PostgreSQL, and set with HttpOnly + SameSite=Lax cookies. Always use Secure cookies + HTTPS in production. Never collect users' Reddit passwords, copy browser session cookies, rotate accounts to evade limits, or claim OAuth exempts your usage from Reddit terms.

The Data API terms prohibit using content to train models without necessary consent. Disclose intended AI processing and secure required permissions for hosted semantic analysis. Build a deletion reconciliation process for content removed from Reddit before public production use.

Useful primary references:
- https://redditinc.com/policies/data-api-terms
- https://support.reddithelp.com/hc/en-us/articles/16160319875092-Reddit-Data-API-Wiki
- https://developers.reddit.com/app-registration
