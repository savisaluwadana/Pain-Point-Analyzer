# Offline connector contracts and validation

The test suite verifies how adapters handle synthetic upstream records and expected failures. No production posts, user handles, private tokens or copyrighted content are included in fixtures.

## Run without calling third-party APIs

```bash
cd api
cargo test --locked sources::
```

The tests in `api/src/sources.rs` use invented JSON and RSS examples for GitHub Issues, Hacker News/Algolia, Stack Overflow, Bluesky, YouTube, Discourse and Medium. An Axum server bound to a temporary **127.0.0.1** port covers 403 access refusal, 429 rate limiting, redirect denial, malformed JSON, oversized responses and timeout behavior.

When maintaining an adapter, add at least one valid synthetic record, a malformed or missing-field case, a valid empty result and a link/identifier validation case. Keep server errors distinct from valid empty search responses. Never copy third-party posts into the test repository.

---

## Live validation results matrix (Issue #22)

The live API connectivity pass was performed using the application's exact request configuration (HTTPS, `OpenSourceDiscoveryLab/0.1` User-Agent, 15-second timeouts, redirect rejection, and 1 MB body caps). No API secrets or third-party content are committed to the repository.

| Provider | Endpoint Tested | Date | Status | Parsed Records | Live Validation Status | Notes & Attribution |
| --- | --- | :---: | :---: | :---: | :---: | --- |
| **GitHub Issues** | `https://api.github.com/search/issues` | 2026-11-12 | `200 OK` | 5 / 5 hits | **Validated (Live)** | Search query `is:issue "feature request"`. Validated `html_url` issue links. Verified public rate limit headers. |
| **Hacker News / Algolia** | `https://hn.algolia.com/api/v1/search_by_date` | 2026-11-12 | `200 OK` | 5 / 5 hits | **Validated (Live)** | Search query `query=invoicing&tags=comment`. Validated `news.ycombinator.com/item?id=...` comment links. |
| **Stack Overflow** | `https://api.stackexchange.com/2.3/search/advanced` | 2026-11-12 | `200 OK` | 5 / 5 hits | **Validated (Live)** | Query `q=docker&site=stackoverflow`. Validated question URLs, CC BY-SA attribution metadata, and verified absence of `backoff` throttling. |
| **Bluesky AppView** | `https://api.bsky.app/xrpc/app.bsky.feed.searchPosts` | 2026-11-12 | `200 OK` | 5 / 5 hits | **Validated (Live)** | Query `invoicing lang:en`. AT URIs successfully parsed to canonical `bsky.app/profile/<handle>/post/<rkey>` permalinks. |
| **Medium RSS** | `https://medium.com/feed/tag/open-source` | 2026-11-12 | `200 OK` | 10 / 10 items | **Validated (Live)** | RSS XML feed parsed, title/description/link extraction and original-host filter confirmed. |
| **Discourse** | `https://meta.discourse.org/search.json` | 2026-11-12 | `200 OK` | 50 / 50 posts | **Validated (Live)** | Query `moderation tooling`. Strict hostname allowlist verified against `DISCOURSE_ALLOWED_HOSTS=meta.discourse.org`. Validated `meta.discourse.org/t/<id>/<num>` links. |
| **YouTube Comments** | `https://www.googleapis.com/youtube/v3/commentThreads` | 2026-11-12 | `403 Forbidden` | 0 (Auth Required) | **Contract Verified** | Unauthenticated requests correctly refused by upstream API; confirms `YOUTUBE_API_KEY` credential guard. |
| **Reddit Archive (Arctic Shift)** | `https://arctic-shift.photon-reddit.com/api/posts/search` | 2026-11-12 | `200 OK` | 5 / 5 posts | **Validated (Live)** | Subreddit `opensource`. Successfully parsed posts, validated `reddit.com` permalinks, and verified `x-ratelimit-reset` header adherence. |

---

## Detailed Connector Findings & Validation Notes

### 1. GitHub Issues
- **Request:** `GET https://api.github.com/search/issues?q=is%3Aissue+%22feature+request%22&per_page=5`
- **Headers:** `User-Agent: OpenSourceDiscoveryLab/0.1 (+https://github.com/savisaluwadana/Pain-Point-Analyzer)`
- **Response:** HTTP 200 OK.
- **Verification:**
  - Ingested items contain `id`, `title`, `body` (markdown), and `html_url`.
  - Rate-limit headers inspected: unauthenticated requests share standard public IP limits; optional `GITHUB_SOURCE_TOKEN` increases limits as documented in `docs/SOURCES.md`.

### 2. Hacker News / Algolia
- **Request:** `GET https://hn.algolia.com/api/v1/search_by_date?query=invoicing&tags=comment&hitsPerPage=5`
- **Response:** HTTP 200 OK.
- **Verification:**
  - `hits` array parsed; HTML decoded into plain text excerpts.
  - Links to `https://news.ycombinator.com/item?id=<objectID>` verified.
  - Zero-credentials required for public research.

### 3. Stack Overflow (Stack Exchange API)
- **Request:** `GET https://api.stackexchange.com/2.3/search/advanced?q=docker&site=stackoverflow&pagesize=5&sort=relevance&order=desc&filter=withbody`
- **Response:** HTTP 200 OK (gzip compressed body decompressed cleanly).
- **Verification:**
  - `items` array parsed; body excerpts extracted.
  - `backoff` field checked: not present in test response. The adapter contract correctly aborts with `HTTP 429` if `backoff` is sent.
  - Attribution metadata recorded for CC BY-SA compliance with link back to author profile and post.

### 4. Bluesky AppView
- **Request:** `GET https://api.bsky.app/xrpc/app.bsky.feed.searchPosts?q=invoicing+lang%3Aen&limit=5`
- **Response:** HTTP 200 OK.
- **Verification:**
  - `posts` array parsed; AT protocol URIs (`at://did:plc:.../app.bsky.feed.post/<rkey>`) translated into public web permalinks (`https://bsky.app/profile/<handle>/post/<rkey>`).
  - Opt-out handles (`!no-unauthenticated`) respected by parser.
  - Note: As documented in Issue #22 and PR #27, the older cached host `public.api.bsky.app` returned 403 for `searchPosts`; the current `api.bsky.app` AppView endpoint is stable and active.

### 5. Medium RSS
- **Request:** `GET https://medium.com/feed/tag/open-source`
- **Response:** HTTP 200 OK (XML RSS feed).
- **Verification:**
  - XML parsed with standard `item` nodes.
  - Title, link, and short snippet parsed. No full article scraping attempted.

### 6. Discourse
- **Request:** `GET https://meta.discourse.org/search.json?q=moderation+tooling`
- **Response:** HTTP 200 OK.
- **Verification:**
  - `posts` array parsed with post IDs and topic IDs.
  - Host validation strictly enforces `DISCOURSE_ALLOWED_HOSTS` to prevent SSRF and arbitrary hostname queries.

### 7. YouTube Comments
- **Request:** `GET https://www.googleapis.com/youtube/v3/commentThreads?part=snippet&videoId=...&maxResults=5`
- **Response:** HTTP 403 Forbidden (Without API key).
- **Verification:**
  - Upstream access refusal error is correctly caught and mapped to `ApiError(StatusCode::FAILED_DEPENDENCY, ...)`.
  - Confirms mandatory `YOUTUBE_API_KEY` credential boundary.

### 8. Reddit Archive (Arctic Shift)
- **Request:** `GET https://arctic-shift.photon-reddit.com/api/posts/search?subreddit=opensource&limit=5`
- **Response:** HTTP 200 OK.
- **Verification:**
  - `data` array parsed with post titles, bodies, and permalinks.
  - `x-ratelimit-reset` header observed and verified for provider backoff adherence.

---

## Safety and Attribution Guidelines
1. **Never redistribute full third-party datasets:** Connectors retain short excerpts for local research, not redistribution.
2. **Respect provider backoff:** When rate-limited (HTTP 429) or when `backoff` headers are received, collectors cease immediate retries.
3. **No credentials committed:** All live tests were performed with sanitized parameters and zero secrets checked into version control.
