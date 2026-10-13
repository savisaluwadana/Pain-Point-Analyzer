# Read-only community source connectors

Open Source Discovery Lab supports seven bounded, source-linked research connectors. **Public access is not unrestricted access.** Each provider retains its own API rules, quotas and content terms. No collector bypasses logins, paywalls or access controls. The primary purpose is local, currently non-commercial exploration of possible open-source contributions.

## Sources and limitations

| Connector | Discovery route | Authentication | Limits / usage notes |
| --- | --- | --- | --- |
| GitHub Issues | [Search Issues REST API](https://docs.github.com/en/rest/search/search) | Public read may work unauthenticated; optional `GITHUB_SOURCE_TOKEN` raises quotas | Search has separate rate limits (unauthenticated often 10/min), including abuse/secondary limits. Requests are read-only. |
| Hacker News | [Algolia HN Search API](https://hn.algolia.com/api) | Public third-party search service | Index maintained independently from the [official HN Firebase API](https://github.com/HackerNews/API). It may be incomplete or change terms; not the official HN keyword-search API. Searches comments by topic. |
| Stack Overflow | [Stack Exchange API](https://api.stackexchange.com/docs) | Public read; per-app keys optional but not required in this version | Rate quotas and dynamic `backoff`. Posts carry CC BY-SA attribution/share-alike requirements; the applicable license version varies with post history. Follow the linked author's post and [licensing documentation](https://stackoverflow.com/help/licensing). |
| Bluesky | [AppView search](https://docs.bsky.app/docs/api/app-bsky-feed-search-posts) via `api.bsky.app` | No key needed today. The cached `public.api.bsky.app` mirror refuses `searchPosts` (HTTP 403), so it is not used for search. Authors labelled `!no-unauthenticated` are skipped. | Bluesky may start requiring authentication for search; an app-password login would then be needed. |
| YouTube comments | [commentThreads.list](https://developers.google.com/youtube/v3/docs/commentThreads/list) | Required `YOUTUBE_API_KEY` | User supplies video ID. Queries one page of top-level comment threads; no automated video search. Quota, caching and deletion rules apply. |
| Discourse | [Discourse API](https://docs.discourse.org/) | Public-search access depends on the forum | Set `DISCOURSE_ALLOWED_HOSTS` to the *exact* public forum hostnames you have permission to search. No private/admin keys, arbitrary URLs or server redirects. |
| Medium RSS | [Medium's published RSS feeds](https://help.medium.com/hc/en-us/articles/214874118-Using-RSS-feeds-of-profiles-publications-and-topics) | Public topic RSS; no Medium API token | Use topic tags such as `open-source`. Read only titles, short descriptions and original links. Medium does not issue new API tokens; the collector cannot fetch paywalled articles. |

## Configure

Generate a separate login key with `openssl rand -hex 32`, then set in local `.env`:

```dotenv
LOCAL_LOGIN_TOKEN=<64-character random hex string>
GITHUB_SOURCE_TOKEN=
YOUTUBE_API_KEY=
DISCOURSE_ALLOWED_HOSTS=meta.discourse.org
SOURCE_RETENTION_DAYS=7
```

Only set credentials that you actually need. Restart the stack with `docker compose up --build -d --wait`. Do not commit `.env` or source API keys. The local login is available only with a localhost `WEB_URL`, and the app is not ready for unrestricted public hosting.

Go to **Community Sources**, choose the source and research query, and collect up to 25 source-linked records per action. YouTube uses an 11-character video ID as its target, and Discourse uses an explicitly allowlisted hostname. Medium expects a lowercase topic tag with hyphens.

## API request

```http
GET /api/sources
POST /api/sources/collect
Content-Type: application/json
Cookie: pil_session=<locally authenticated session>
Origin: http://localhost:3000

{"source":"github","query":"missing feature","project_id":null,"limit":15}
```

`source` values: `github`, `hackernews`, `stackoverflow`, `bluesky`, `youtube`, `discourse`, `medium`. The POST endpoint requires a valid session and same-origin protection. Responses return `saved` and `matched` counts. Evidence is deduplicated within each account/source.

### Data and safety

- API requests use fixed hostnames except Discourse domains explicitly allowlisted in the local environment.
- Fetches use HTTPS, reject redirects, have 15-second timeouts and a 1 MB response cap.
- A PostgreSQL budget limits each owner/source to four collection requests per minute. The API does *not* retry aggressively after rate limiting; observe the provider's own `429` and backoff requirements.
- Only short excerpts are kept; provider-supplied HTML is stripped for display. Content and extracted phrases are evidence, never instructions for an AI host.
- Source URLs, source-specific IDs, timestamps, and basic attribution metadata are retained. Never redistribute sources as a dataset or treat this repository's MIT software license as rights to third-party content.
- Connected-source excerpts are removed after seven days by default (`SOURCE_RETENTION_DAYS`, 1–30 days). Consider provider-specific rules and deletion requirements before operational use, especially YouTube/Bluesky content that is removed at the source.
- Respect copyright, licensing, robots/rate limits where applicable, and each community's terms. RSS isn't consent for arbitrary full-article scraping.
- Successful CI checks mock account sessions and validate authorization/requests. Live third-party collection still requires provider-side acceptance testing; external platforms can change quotas and response formats without notice.

### Known limitations

Collection currently uses one page per click; no provider pagination or scheduled refresh. HN full-text search relies on Algolia's third-party index rather than Firebase. YouTube requires selecting a known video ID. Medium supports topic-tag feeds but not arbitrary full-text article search. Discourse public search varies by forum. Substantial provider API failures are surfaced as an error instead of silently falling back to scraping.
