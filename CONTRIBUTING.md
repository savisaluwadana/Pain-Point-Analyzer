# Contributing to Open Source Discovery Lab

Thanks for helping build a community-first, open-source research tool. This project is in **alpha**. Small, well-tested improvements and honest reports about broken workflows are especially useful.

## Where to begin

1. Read the [README](README.md), [five-minute demo](docs/DEMO.md), and [roadmap](docs/ROADMAP.md).
2. Check [open issues](https://github.com/savisaluwadana/Pain-Point-Analyzer/issues). If you have an idea or found a bug, open an issue using a template before a large change.
3. For your first PR, prefer documentation improvements, better error handling, connector tests, or accessibility fixes.
4. In your PR, explain the **user-visible problem**, the approach taken, tests performed, and any data permissions affected.

## Local setup

Requires Rust stable, Node.js 22+, Docker Compose v2, OpenSSL, curl and jq.

```bash
cp .env.example .env
# Set a different `openssl rand -hex 32` result for each:
# TOKEN_ENCRYPTION_KEY_HEX and LOCAL_LOGIN_TOKEN
# Also set POSTGRES_PASSWORD.
docker compose up --build -d --wait
bash scripts/try-demo.sh
```

The demo uses **fictional observations**. Keep credentials in local `.env` (gitignored), not in commits or test output.

## Tests

```bash
(cd api && cargo check --all-targets && cargo test)
(cd web && npm install --no-audit --no-fund && npm run typecheck && npm run build)
docker compose config --quiet
bash scripts/test_mcp.sh
bash scripts/try-demo.sh
```

The last two scripts require the Docker stack running and the environment described in [the demo](docs/DEMO.md). `test_mcp.sh` is primarily a CI integration harness that creates and deletes fixture accounts; **run it only in a disposable test database**, not your personal research installation.

CI verifies the main local Rust/Next.js/PostgreSQL/MCP behavior. For provider connector changes, include parsing, error, limit and authorization tests and, where you have appropriate access, manual provider smoke-test notes. Never introduce scraping workarounds or hard-code API credentials to make a test pass.

## Engineering and data guidelines

- **Source-first:** Preserve source URLs and attribution. Avoid presenting heuristic clusters as ground truth or measured demand.
- **Provider rules:** Read the upstream API terms, quotas, retention and licensing before connecting a new source. A public page or RSS feed isn't authorization to collect everything.
- **Privacy:** Do not commit personal data, scraped datasets, tokens, private API responses or real user comments as fixtures. Prefer original synthetic fixtures.
- **Security:** Bound requests, inputs and response sizes. Never allow arbitrary URL fetching or unreviewed credential-bearing redirects.
- **Keep changes scoped:** Avoid unrelated formatting or rewriting old application modules without reason.
- **Accessibility:** Use semantic controls, labels, keyboard navigation and clear errors.
- **External data:** Third-party source content does not inherit the repository's MIT code license.

## Submitting pull requests

Fork the repository, make a topic branch, push it and open a PR against `main`. Complete the PR checklist; wait for CI. For a behavior change include tests and documentation updates. Review and merge timings are best effort; this is a community-maintained alpha.

See the [Code of Conduct](CODE_OF_CONDUCT.md) and [Security Policy](SECURITY.md).
