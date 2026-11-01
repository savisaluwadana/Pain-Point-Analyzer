# Offline connector contracts and validation

The test suite verifies how adapters handle synthetic upstream records and expected failures. No production posts, user handles, private tokens or copyrighted content are included in fixtures.

## Run without calling third-party APIs

```bash
cd api
cargo test --locked sources::
```

The tests in `api/src/sources.rs` use invented JSON and RSS examples for GitHub Issues, Hacker News/Algolia, Stack Overflow, Bluesky, YouTube, Discourse and Medium. An Axum server bound to a temporary **127.0.0.1** port covers 403 access refusal, 429 rate limiting, redirect denial, malformed JSON, oversized responses and timeout behavior.

When maintaining an adapter, add at least one valid synthetic record, a malformed or missing-field case, a valid empty result and a link/identifier validation case. Keep server errors distinct from valid empty search responses. Never copy third-party posts into the test repository.

## Live validation matrix

This PR **does not claim** live third-party connectivity. Open issue [#22](https://github.com/savisaluwadana/Pain-Point-Analyzer/issues/22) requires a separately authorized provider-by-provider acceptance pass.

| Provider | Offline coverage | Live result |
| --- | --- | --- |
| GitHub Issues | Synthetic parser, payload validation, HTTP failures | Not verified in this PR |
| Hacker News / Algolia | Synthetic parser, payload validation | Not verified in this PR |
| Stack Overflow | Synthetic parser, license metadata and backoff | Not verified in this PR |
| Bluesky | Synthetic search, reply ancestry, opt-outs, paging query construction and HTTP failures | Not verified in this PR |
| YouTube | Synthetic parser and ID validation | Requires authorized API key |
| Discourse | Synthetic parser and allowlist validation | Requires permitted host |
| Medium RSS | Synthetic XML parser, excerpt and original-host filtering | Not verified in this PR |

Record a live test date, provider, response status, sanitized query or permitted forum, parsed count, original link verification and redacted failure details before marking a provider validated. Do not attach real content, credentials or tokens.
