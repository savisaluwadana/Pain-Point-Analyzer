# Release notes — v0.2.0-alpha.1

**Tag:** `v0.2.0-alpha.1`  
**Status:** Community Alpha Prerelease  
**Target:** `main` (`febee81de005d2e48c2b2888a6780dd676b80938`)  
**Legacy notice:** The older `v-1.0` GitHub release from September 2026 is a legacy prototype artifact predating this architecture. Do not treat `v-1.0` as the current or stable version.

---

## Overview

Open Source Discovery Lab (`v0.2.0-alpha.1`) is a self-hosted, community-driven research workspace for discovering recurring developer challenges and inspiring useful open-source software projects. It operates on bounded, source-linked observations with full respect for platform terms, licensing, and user privacy.

---

## What this alpha includes

### 1. Dual Deployment Architecture
- **Docker / Web edition:** Rust Axum API, Next.js 15 dashboard, and PostgreSQL 17 storage.
- **Desktop edition foundation:** Tauri 2 native window with SQLite local storage, ad-hoc macOS signing, and dedicated frontend export.

### 2. Eight Read-Only Community Connectors
- **GitHub Issues:** Search issues by topic with rate-limit tracking.
- **Hacker News / Algolia:** Search public community comments.
- **Stack Overflow:** Relevance-sorted question discovery with CC BY-SA author attribution and dynamic backoff handling.
- **Bluesky AppView:** Read-only post search with AT URI canonicalization, author opt-out filters, and bounded reply thread enrichment.
- **Medium RSS:** Public topic feed exploration.
- **Discourse:** Domain-allowlisted public forum searches (`DISCOURSE_ALLOWED_HOSTS`).
- **YouTube:** API key-gated comment thread searches.
- **Reddit Archive (Arctic Shift):** Privacy-preserving archive post retrieval with pacing and rate-limit reset handling.

### 3. Transparent Heuristic Analysis
- Deterministic `keywords-v2` rule-based triage.
- Within-run relative priority calculation based on independent story count, source family diversity, and complaint strength.
- Explicit caveats and insufficient-evidence notices when fewer than three independent stories support a theme.

### 4. Local-First Security and Privacy
- Single-operator localhost authentication (`LOCAL_LOGIN_TOKEN`).
- Source text is never copied into durable reports; reports retain only cryptographic content hashes and IDs, with on-read quotation hydration.
- Bounded retention periods (`SOURCE_RETENTION_DAYS`, `REDDIT_RETENTION_DAYS`).

### 5. Reproducible Synthetic Demo & MCP Tooling
- `bash scripts/try-demo.sh` runs a complete 5-minute local demo with 6 invented observations, requiring zero network calls or credentials.
- Rust Model Context Protocol (MCP) server providing 58 research tools for local AI assistants.

---

## Live Connectivity Validation Matrix

All public connectors have been smoke-tested against live upstream APIs (see [docs/CONNECTOR_TESTS.md](CONNECTOR_TESTS.md)):

| Provider | Endpoint | Live Status | Notes |
|---|---|:---:|---|
| GitHub Issues | `api.github.com/search/issues` | **Validated** | Ingests issue titles, excerpts, and URLs. |
| Hacker News / Algolia | `hn.algolia.com/api/v1/search_by_date` | **Validated** | Resolves comment permalinks. |
| Stack Overflow | `api.stackexchange.com/2.3/search/advanced` | **Validated** | Preserves CC BY-SA licensing and checks `backoff`. |
| Bluesky AppView | `api.bsky.app/xrpc/app.bsky.feed.searchPosts` | **Validated** | Translates AT URIs to `bsky.app` web links. |
| Medium RSS | `medium.com/feed/tag/<tag>` | **Validated** | Parses XML channel items without full-page scraping. |
| Discourse | `<host>/search.json` | **Validated** | Host allowlist enforced against `meta.discourse.org`. |
| YouTube Comments | `googleapis.com/youtube/v3/commentThreads` | **Contract Verified** | Refuses unauthenticated calls with HTTP 403. |
| Reddit Archive | `arctic-shift.photon-reddit.com/api/posts/search` | **Validated** | Confirmed query parsing and rate-limit reset headers. |

---

## Known Limitations

- **Community Alpha:** Intended for local exploration by individual developers and researchers; not certified for multi-tenant hosting.
- **Heuristic, not demand forecasting:** Keyword clustering surfaces hypotheses, not statistically verified market demand.
- **External provider variability:** Upstream platforms can change rate limits, authentication policies, or schemas at any time.

---

## Verification & Publishing Checklist

- [x] All post-merge CI workflows green on `main` (`febee81`)
- [x] Synthetic demo screenshot and walkthrough added to `README.md` and `docs/DEMO.md` (#23)
- [x] Live provider connectivity acceptance matrix documented (#22)
- [x] Prerelease draft prepared with legacy release disclaimer
- [x] Tag `v0.2.0-alpha.1` configured as a GitHub prerelease
