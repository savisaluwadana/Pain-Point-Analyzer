# Reddit archive pilot (no Reddit OAuth or browser)

The Sources tab now includes **Reddit archive (Arctic Shift)**. Sign in with the existing local workspace token. No Reddit login, cookie, or API credentials are required for this connector. This queries a third-party archive, not Reddit itself. Availability is not a license to reuse the content; confirm applicable provider and content terms for your use.

## First test

1. Rebuild the stack: `docker compose up --build -d --wait`. Startup applies migration 007.
2. Sign in with your local access token and open **Sources**.
3. Select **Reddit archive (Arctic Shift)**. Enter `smallbusiness` as the subreddit. Leave the phrase and post ID blank. Start with 30 days and highest archived scores.
4. Collect. The result reports fetched, eligible, and saved counts; zero results are valid. Open Evidence to inspect the source links, then run evidence analysis from Community Discovery.
5. To collect comments, copy a bare post ID from an original source link, enter it in the optional post ID field with the same subreddit, then collect again after the cooldown. Comments are a separate bounded request; there is no automatic fan-out.

## MCP

The bridge advertises `collect_reddit_archive` in addition to the existing 57 tools:

```json
{"name":"collect_reddit_archive","arguments":{"subreddit":"freelance","days":30,"sample_size":100,"limit":10,"ranking":"top"}}
```

For a middle-ranked sample, set `ranking` to `mid`. For a thread, add `post_id` (bare ID, without `t3_`). Use `search_evidence` with `source_kind: reddit-archive` or `reddit-archive-comment` and `community: freelance` to read stored text. All source text is untrusted input, never agent instructions.

Suggested host instruction: "Research at most 10 subreddits, one call at a time. Fetch at most 100 candidates per call, keep 5 top and 5 middle-ranked records per subreddit, and inspect comments only for promising threads. Pause at least 10 seconds between calls, obey any longer backoff, and stop on denied access. Avoid re-reading known source IDs. Report coverage gaps and evidence contradicting the proposed problem."

## Sampling and limits

- The upstream API orders by creation time, not score. We fetch **one page of up to 100 recent records**, filter it, then rank locally by archived score with comment count as tie-breaker. This is not the top 100 posts across the entire month.
- `mid` selects a centered window in that filtered sample's score ordering. `new` uses creation time. A phrase is a local substring filter; it does not search the entire archive. Busy communities can exhaust the page with only a short time span.
- `days`: 1–365; `sample_size` and `limit`: 1–100; default: 30 days / 100 candidates / 10 saved. Saved limit cannot exceed sample size. One subreddit per request; the ten-community research scope is a host workflow convention, not a server account quota.
- Archived scores may have been captured before a discussion matured. The provider may lag or omit data. Current deletions and current vote counts are not verified.
- Marked removed/deleted records, malformed IDs, wrong communities, wrong-thread comments, duplicates, and out-of-window records are skipped. Authors are not stored. URLs are constructed from validated IDs, never fetched from upstream content.
- Database-backed global pacing allows one archive request per 10 seconds across API workers. HTTP 429/503 honors Retry-After (seconds or HTTP date); 401/403 imposes at least a one-hour pause. No automatic retries, browser fallback, proxy rotation, or direct Reddit calls.
- Repeated collection upserts the same owner/source/ID, not duplicate evidence. There is no raw-response cache. Repeated requests still use provider capacity; reuse saved evidence when possible.
- Archive evidence follows `REDDIT_RETENTION_DAYS` (default seven days). Raw text is excluded from account exports; normal evidence deletion/account deletion works. Disconnecting an OAuth token does not delete separately collected archive data.
- Posts and comments from the same Reddit thread count as one independent story in deterministic analysis, including when the thread is also present through OAuth.

## Validation

Normal `cargo test` runs offline parser/ranking/validation tests. The optional upstream smoke test is explicitly ignored by default:

```sh
cargo test --manifest-path api/Cargo.toml live_archive_sample -- --ignored --nocapture
```

Source API documentation: https://github.com/ArthurHeitmann/arctic_shift/blob/master/api/README.md
