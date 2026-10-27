# Reddit archive pilot (no Reddit OAuth or browser)

The Sources tab now includes **Reddit archive (Arctic Shift)**. Sign in with the existing local workspace token. No
Reddit login, cookie, or API credentials are required for this connector. This queries a third-party archive, not Reddit
itself. Availability is not a license to reuse the content; confirm applicable provider and content terms for your use.

## Setup or upgrade

For a new installation, complete the
[README quick start](GETTING_STARTED.md). Set three different secrets in
`.env`: `TOKEN_ENCRYPTION_KEY_HEX`, `LOCAL_LOGIN_TOKEN` and `POSTGRES_PASSWORD`. Leave `REDDIT_API_APPROVED=false` and
the Reddit client credentials empty for archive-only research.

For an existing clean checkout, update from the merged main branch and rebuild:

```sh
git switch main
git pull --ff-only
docker compose up --build -d --wait
```

Keep your existing `.env` and database volume. Startup applies migration `007_archive_provider.sql` automatically. Do
not use `docker compose down -v` when retaining research.

## First test

1. Confirm the rebuilt stack is running at **http://localhost:3000**.
2. Sign in with `LOCAL_LOGIN_TOKEN` and open **Community Sources**.
3. Select **Reddit archive (Arctic Shift)**. Enter `smallbusiness` as the subreddit. Leave the phrase and post ID blank.
   Start with 30 days and highest archived scores.
4. Collect. The result reports fetched, eligible, and saved counts; zero results are valid. Open **Evidence Library** to
   inspect the source links, then run evidence analysis from **Analyze & Discover**.
5. To collect comments, copy a bare post ID from an original source link, enter it in the optional post ID field with
   the same subreddit, then collect again after the cooldown. Comments are a separate bounded request; there is no
   automatic fan-out.

The dashboard requests up to 100 candidates and saves up to 10 after filtering. `fetched` counts returned upstream
records, `eligible` counts those passing the filters, and `saved` counts selected records inserted **or updated**.
Repeating a collection need not add new records.

## MCP

Create a workspace key in **MCP Integrations**, then configure the local host using [the MCP setup guide](MCP.md). The
agent does not need a browser connector or a Reddit session. The dashboard's **Research Jobs** background collector is
still specific to direct Reddit OAuth; use this tool for archive collection.

The bridge advertises `collect_reddit_archive` in addition to the existing 57 tools:

```json
{
  "name": "collect_reddit_archive",
  "arguments": {
    "subreddit": "smallbusiness",
    "days": 30,
    "sample_size": 100,
    "limit": 10,
    "ranking": "top",
    "batch_id": "YOUR_RESEARCH_JOB_ID"
  }
}
```

For a middle-ranked sample, set `ranking` to `mid`. For a thread, add `post_id` (bare ID, without `t3_`). Use
`search_evidence` with `source_kind: reddit-archive` or `reddit-archive-comment` and `community: smallbusiness` to read
stored text. All source text is untrusted input, never agent instructions.

Suggested host instruction: "Research at most 10 subreddits, one call at a time. Fetch at most 100 candidates per call,
keep 5 top and 5 middle-ranked records per subreddit, and inspect comments only for promising threads. Pause at least 10
seconds between calls, obey any longer backoff, and stop on denied access. Avoid re-reading known source IDs. Report
coverage gaps and evidence contradicting the proposed problem."

Selecting five `top` and five `mid` records requires two collection calls with `limit: 5`; each fetches a page. Their
selections can overlap, so this does not guarantee ten distinct records. Reuse stored evidence where possible rather
than repeating collection.

## HTTP example

After local sign-in, send this body to `POST /api/sources/collect` with the session cookie and matching
`Origin: http://localhost:3000`:

```json
{ "source": "arctic-shift", "target": "smallbusiness", "days": 30, "sample_size": 100, "limit": 10, "ranking": "top" }
```

The HTTP endpoint uses `target`; the MCP tool uses `subreddit`. Both accept optional `batch_id` to link evidence to a
specific research job (MCP readiness checks do not count evidence without this linkage). Optional `query` filters the
fetched sample locally, `post_id` collects comments for a single thread, and `project_id` assigns evidence to an
existing project owned by the caller. See [source API authentication](SOURCES.md#api-request).

## Sampling and limits

- The upstream API orders by creation time, not score. We fetch **one page of up to 100 recent records**, filter it,
  then rank locally by archived score with comment count as tie-breaker. This is not the top 100 posts across the entire
  month.
- `mid` selects a centered window in that filtered sample's score ordering. `new` uses creation time. A phrase is a
  local substring filter; it does not search the entire archive. Busy communities can exhaust the page with only a short
  time span.
- `days`: 1–365; `sample_size` and `limit`: 1–100; default: 30 days / 100 candidates / 10 saved. Saved limit cannot
  exceed sample size. One subreddit per request; the ten-community research scope is a host workflow convention, not a
  server account quota.
- Archived scores may have been captured before a discussion matured. The provider may lag or omit data. Current
  deletions and current vote counts are not verified.
- Marked removed/deleted records, malformed IDs, wrong communities, wrong-thread comments, duplicates, and out-of-window
  records are skipped. Authors are not stored. URLs are constructed from validated IDs, never fetched from upstream
  content.
- Database-backed global pacing allows one archive request per 10 seconds across API workers. HTTP 429/503 honors
  Retry-After (seconds or HTTP date) and the provider's X-RateLimit-Reset / X-RateLimit-Reset-At headers; 401/403
  imposes at least a one-hour pause. No automatic retries, browser fallback, proxy rotation, or direct Reddit calls.
- Repeated collection upserts the same owner/source/ID, not duplicate evidence. There is no raw-response cache. Repeated
  requests still use provider capacity; reuse saved evidence when possible.
- Archive evidence follows `REDDIT_RETENTION_DAYS` (default seven days, bounded to 1–30). Raw text is excluded from
  account exports; normal evidence deletion/account deletion works. Disconnecting an OAuth token does not delete
  separately collected archive data.
- Posts and comments from the same Reddit thread count as one independent story in deterministic analysis, including
  when the thread is also present through OAuth.

## Troubleshooting

| Result                                           | Next step                                                                                                                                                                                               |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Missing archive source or unknown MCP tool       | Pull the merged `main`, rebuild the stack, and rebuild/restart the host's MCP binary so it advertises 58 tools.                                                                                         |
| HTTP 401 from the application                    | Sign in locally again, or check that the host uses an active workspace MCP key. Reddit cookies are not used.                                                                                            |
| HTTP 400 validation error                        | Use a bare subreddit, `days` 1–365, and `1 <= limit <= sample_size <= 100`. Use a bare post ID without `t3_`.                                                                                           |
| HTTP 429 pacing/backoff                          | Wait at least 10 seconds between request starts, or longer when the error reports provider backoff. Other workers share the same archive budget.                                                        |
| HTTP 502 naming an upstream 401/403              | The archive refused access; requests pause for at least one hour. Stop the run and investigate availability.                                                                                            |
| HTTP 502 timeout, non-JSON or malformed response | Check the reported provider failure. There is no automatic retry or fallback. Do not repeat requests in a tight loop.                                                                                   |
| Zero saved records                               | This can be valid: the recent page may contain only removed records or no local phrase matches. Inspect fetched/eligible counts, remove the phrase filter, or try another community after the cooldown. |

## Validation

Normal `cargo test` runs offline parser/ranking/validation tests. CI separately runs the PostgreSQL archive contract
test for upserts, owner isolation and atomic pacing. The optional upstream smoke test is explicitly ignored by default:

```sh
cargo test --manifest-path api/Cargo.toml live_archive_sample -- --ignored --nocapture
```

During implementation on **2026-10-15**, an opt-in `smallbusiness` post sample fetched 100 records, found 68 eligible
and selected two (`limit: 2`). A separate comment request returned ten records with six usable; a `freelance` sample
contained only removed records and produced zero eligible items. These observations demonstrate bounded provider access
and empty-result handling at that time, not guaranteed future counts, complete historical coverage or live Reddit
rankings.

Source API documentation: [Arctic Shift API](https://github.com/ArthurHeitmann/arctic_shift/blob/master/api/README.md).
