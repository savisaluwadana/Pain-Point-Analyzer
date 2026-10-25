# Analysis quality and operating limits

## Implemented

- `keywords-v2` uses word/phrase boundaries, normalizes curly apostrophes and handles limited clause-local negation.
  `terror`, neutral `pricing`, `not slow`, and `no errors` do not create complaints. `not only slow` remains positive.
- Eight explicit rule categories remain. Unclassified records are counted; paraphrases, irony and complex negation
  can still be missed. Manual-work terms are proxies, not proof that a task is unwanted.
- Union-find merges provider threads and normalized exact copies of at least 100 characters transitively. Sorting by
  evidence UUID makes grouping and output stable across input permutations. Reddit direct/archive forms share thread
  identities; HN stories, Discourse host/topic pairs and YouTube videos use stored parent metadata when available.
  Missing metadata falls back to the source record. Similar paraphrases are not deduplicated.
- Each group counts once. A copied group gets one canonical provider family from its lowest-UUID member, so cross-posts
  cannot inflate source diversity. This is an explicit deterministic convention, not a credibility assessment.
- One to two supporting discussions are labeled insufficient evidence. They remain visible for investigation.

## Ranking version 2

`raw_priority = 0.70 * ln(1 + T) + 0.15 * ln(1 + D) + 0.15 * C`

`T` is independent supporting stories; `D` is canonical provider families; `C` is mean per-story complaint strength.
Strength is the largest member's distinct positive phrase count, capped at four and divided by four. Recency is not
part of this version's formula. Raw priority grows logarithmically rather than clipping at 100.

`severity_score` is retained for API compatibility but now contains a within-run midrank percentile. Ties get equal
values. A single theme gets 50 because there is no comparative ranking. Neither number is buying intent, statistical
confidence or market size. Do not compare percentiles across runs. Reports lacking `score_version: 2` retain their
legacy interpretation; no historical reports or immutable migrations are rewritten.

## Coverage and privacy

HTTP analysis defaults to 1,000 rows; MCP analysis defaults to 100. Both allow an explicit limit up to 10,000.
Stable date/UUID ordering and window counts expose selected/omitted records from the same SQL snapshot. HTTP scope
uses a lexical topic filter; MCP uses an explicit literal search plus source/community/batch/date filters.
`since` means collection time, not publication time. Unsupported tag filters fail clearly instead of being ignored.
Analysis runs on the blocking pool with one global permit; concurrent requests get 429 and can retry later.

Reports persist evidence IDs and content hashes, never raw supporting excerpts. When read, up to five representative
references per category are resolved against the current owner's evidence. Changed, deleted or expired evidence is
marked unavailable; excerpts are limited to 240 characters. Normal exports retain reference-only stored reports.
Representative excerpts currently use the beginning of the selected record, not a model-selected sentence.

## Verification

Rust regressions cover lexical traps, negation, transitive copy/thread grouping, stable ordering, sparse evidence,
non-saturating raw ranking and report-text exclusion. Offline provider tests cover JSON container shape changes,
synthetic parser records, bad IDs, attribution and bounded excerpts. Existing permission/rate-limit tests remain.
Frontend tests cover legacy reports, coverage warnings, supporting excerpts, unavailable evidence and unsafe links.
PostgreSQL integration checks cover real date filtering, truncation, stored-report text exclusion, content changes,
deletion and cross-owner reference isolation. These checks do not certify every live external provider.

Run:

```bash
(cd api && cargo fmt --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked)
(cd web && npm ci && npm run lint && npm run format:check && npm run typecheck && npm test && npm run build)
docker compose up --build -d --wait
bash scripts/test_mcp.sh # test database only: inserts/deletes synthetic test accounts
bash scripts/try-demo.sh
```

## Synthetic workload timing

An opt-in test generates unique, short fictional records and checks that none are collapsed accidentally:

```bash
cd api
cargo test --locked analysis_workload_benchmark -- --ignored --nocapture
```

On the development container (x86_64, AMD EPYC 9V74 reported by the host), the unoptimized debug build measured
821 ms for 1,000 records and 8,175 ms for 10,000 records. These are short-input scorer timings, excluding database,
network and UI time. They are not a load test, a quality benchmark or a promise for long inputs/other machines.
Project candidates are listed newest-run-first; scores from different runs are not globally ranked.

## Still required before claiming semantic or production readiness

The embedding engine and semantic evaluation described in ANALYSIS_DESIGN.md are not shipped. Live-provider testing,
manual accessibility review, load/security review and release publication remain separate gates. This application is
still a local single-operator alpha. No paid services, accounts or credentials were added by these changes.
