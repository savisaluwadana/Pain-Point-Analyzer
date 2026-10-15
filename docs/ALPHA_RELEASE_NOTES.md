# Draft release notes — v0.2.0-alpha.1

**Status: release draft, not yet a published tag.** These notes describe the current alpha branch after merge. Publish them as a GitHub prerelease only after merging and validating the selected commit. The old `v-1.0` release from September 2026 describes an earlier codebase and should not be used as evidence that this iteration is stable.

## What this alpha includes

- Self-hosted Rust API, Next.js dashboard and PostgreSQL storage.
- Eight read-only community source adapters: Arctic Shift Reddit archives, GitHub Issues, HN/Algolia comments, Stack Overflow questions, Bluesky posts, YouTube comments, allowlisted Discourse and Medium topic RSS.
- Optional localhost-only token login independent of Reddit credentials.
- Manually saved research evidence, rule-based theme summaries, community project candidates and source links.
- Optional Rust MCP bridge with 58 tool schemas, including browser-free `collect_reddit_archive`.
- Archive top/mid/new selection within a recent sample, shared pacing, explicit provenance and short Reddit evidence retention.
- Direct Reddit API 401/403 denials stop background retries instead of repeatedly consuming attempts.
- A fully synthetic, six-item demo executable with no external API account.
- MIT licensing, contributor guidance, security policy and public alpha issue templates.

## Start here

Read [README](../README.md), then follow [Five-minute demo](DEMO.md) with `bash scripts/try-demo.sh`. For Reddit research without OAuth, follow the [Arctic Shift quick start](ARCTIC_SHIFT.md). Use [Source connectors](SOURCES.md) for other providers.

## Known limitations

- **No live provider certification.** GitHub Actions uses a local database and fixture sessions; it does not guarantee public API permissions or the behavior of all eight source connectors. A small Arctic Shift live sample passed during implementation; this does not establish complete coverage or ongoing availability.
- Research is **heuristic keyword analysis**, not proof of demand or semantic inference by the Rust service. Optional MCP-host reasoning is a separate workflow.
- Source collection is a single page per click; there is no scheduled multi-provider ingestion.
- The direct Reddit API connector requires official approval. Archive collection, local research and the demo do not require Reddit OAuth. Archive data can lag, and its scores are not live subreddit rankings.
- Local login is single-operator/localhost only. **Do not expose the local token login publicly.**
- Third-party content, licenses, platform quotas and deletion responsibilities are separate from our MIT source-code license.
- No independent penetration test, live cross-browser acceptance test or guaranteed uptime.

## Feedback wanted

Can you complete the first-run demo in under ten minutes? Does the evidence view make provenance clear? Which connector works (or fails) for you? Is the distinction between a recurring complaint and a validated open-source contribution clear? Please use the public issue templates.

## Publishing checklist

- [ ] All four CI jobs green on the release commit
- [ ] Smoke-test local login, synthetic demo and at least one real authorized provider where available
- [ ] Confirm README and docs match actual release behavior
- [ ] Create tag `v0.2.0-alpha.1` on the verified commit, then create GitHub **prerelease** using these notes
- [ ] Share as a call for alpha testers, not as a stable release
