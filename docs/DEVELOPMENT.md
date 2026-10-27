# Development and testing

This guide covers the source repository, local iteration, CI and
contribution conventions. The application is a local alpha, not a
production-hosted SaaS.

## Repository map

```text
api/           Rust Axum API, collectors, scoring, MCP bridge and worker
web/           Next.js 15, React 19, client workspace and Vitest tests
db/            PostgreSQL migrations (append-only, numbered)
docs/          Public documentation source and site styles
scripts/       Offline demo and PostgreSQL-backed integration suite
.github/       Continuous integration, project templates, Pages publishing
mkdocs.yml     Public documentation navigation and theme
compose.yaml   Local web + api + postgres services
Dockerfile.*   Runtime builds
```

## Toolchain

- Rust stable, `cargo` and `rustfmt` / Clippy
- Node.js 22 and npm with the committed lockfile
- PostgreSQL 17, generally via Docker Compose
- Docker Engine and Compose v2
- Python 3.12 and `mkdocs-material` **only** to preview the documentation website

## Start services and run tests

```bash
cp .env.example .env
# Generate and paste three independent secrets as described in the install guide.
docker compose up --build -d --wait

(cd api && cargo fmt --check)
(cd api && cargo clippy --all-targets --locked -- -D warnings)
(cd api && cargo check --all-targets --locked && cargo test --locked)

(cd web && npm ci --no-audit --no-fund)
(cd web && npm run lint && npm run format:check && npm run typecheck)
(cd web && npm test && npm run build)

bash scripts/test_mcp.sh   # Test database only; inserts and deletes synthetic accounts
bash scripts/try-demo.sh
```

Run the integration script against the **disposable** test environment
described in `.github/workflows/ci.yml`. It may create and delete local test
accounts and records. Do not point it at important research data.

The CI pipeline includes Rust, frontend, Docker Compose and integration
jobs. External API availability, provider authorization and public
production security are **not** certified by these tests.

## Backend contribution areas

| Module | Responsibility |
| --- | --- |
| `main.rs` | Configuration checks, routing, DB startup and workers |
| `oauth.rs` / `local_auth.rs` | OAuth and localhost-only browser sessions |
| `csrf.rs` | Same-origin mutation restrictions |
| `sources.rs` / `archive.rs` / `reddit.rs` | Source-specific read-only connectors and budgets |
| `scoring.rs` | Deterministic grouping, classification and priority scores |
| `research.rs` | Evidence, projects, runs, exports and deletion |
| `opportunity_review.rs` | Owner-scoped brief hydration and manual review |
| `jobs.rs` | Leased approved Reddit collection worker and cleanup |
| `mcp/...` | MCP tool schemas, local bridge, keys and research state |

When adding a connector, verify source terms and quotas before writing a
fetcher. Enforce host allowlists, timeouts, response-size caps, errors that
differentiate 403 from 429, attribution and deletion/retention requirements.
No login bypasses, rotating credentials, or arbitrary scraping fallbacks.

When changing analysis logic, add deterministic fixture tests, document
the version and limitations, preserve evidence grouping, and compare
against a held-out evaluation set. Do not silently rewrite historic
reports or enable remote model calls by default.

## Frontend and API changes

Keep the authentication boundary in Rust. The Next.js `/api/*` path is
a same-origin rewrite to the internal API. UI tests live in `web/tests`
and mock transport while exercising real components and workspace
state. Extend these tests for navigation, unauthorized responses, unsafe
source links, persistence and error states.

Migration rules: append a new `db/NNN_description.sql` file; do not
change migrations already applied to another installation. Preserve
account ownership checks and data export/delete coverage for new fields.

## Documentation preview

```bash
python3 -m venv .venv-docs
. .venv-docs/bin/activate
python -m pip install -r docs/requirements.txt
python -m mkdocs serve
# Open http://127.0.0.1:8000
python -m mkdocs build --strict
```

`mkdocs build --strict` is also required by CI. Doc edits should add
context to these pages instead of duplicating instructions in unrelated
README sections.

## Review and release process

1. Open a focused issue or discuss the intended change.
2. Branch from `main`; keep application and documentation changes scoped.
3. Include relevant unit, integration, or snapshot fixtures using synthetic data.
4. Document feature availability, security tradeoffs and backward compatibility.
5. Open a PR and inspect CI, a local setup, the original sources and review feedback.
6. Do not claim a release, stable API or public deployment until separately verified.

See [CONTRIBUTING.md](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/CONTRIBUTING.md),
[roadmap](ROADMAP.md), [operations](OPERATIONS.md) and
[security guidance](SUPPORT_AND_SECURITY.md).
