# Community alpha roadmap

The goal is to help developers identify **verifiable community needs** and choose between contributing to existing
open-source projects or building something genuinely missing. This is a collaborative roadmap, not a claim that future
features already exist.

## Community alpha — in scope now

- [x] MIT software license and current Rust, Next.js and PostgreSQL stack
- [x] Local opt-in token login; Reddit independently optional
- [x] Eight manual, single-page read-only source adapters (subject to provider access)
- [x] Arctic Shift archive collection without Reddit OAuth through the dashboard and MCP; bounded samples and shared
      backoff
- [x] Reddit post/comment grouping by thread across direct and archive evidence
- [x] Evidence library, source links, rule-based theme summaries, project ideas
- [x] Owner-scoped manual opportunity review with live evidence hydration and user-entered validation notes
- [x] Reproducible **fictional** demo with CI coverage
- [x] Contributor, conduct, security, support and issue templates
- [ ] Run and record **real provider smoke tests** for every advertised connector (credentials where needed)
- [ ] Manual cross-browser, keyboard-only and accessibility review
- [ ] Independent security review of auth, data permissions, retention and backups
- [ ] Create annotated GitHub alpha tag and publish release notes

## Near-term candidate contributions

1. **Provider contract tests:** API response fixtures, parsing, 401/403/429, timeout and delete behavior without calling
   real providers in CI.
2. **Better evidence grouping:** cross-platform near-duplicate detection, thread grouping for more providers,
   confidence/caveat display.
3. **Source health diagnostics:** distinguish a configured connector from a successfully verified live provider
   connection.
4. **Onboarding and interface:** better source selection, sample screenshots/short screencast, mobile/keyboard testing
   and clearer empty states.
5. **Attribution/compliance:** source-specific attribution and deletion policy tests beyond the existing Stack Overflow
   attribution panel.
6. **Research-to-contribution:** search existing GitHub projects before proposing new ones; create GitHub-ready
   contribution proposals.
7. **Safer production architecture:** per-user authentication, scoped provider keys, queued requests and observability
   before any public hosting.

## Milestone definitions

**Alpha:** A developer can clone, configure local login, run the fictional demo and inspect a source-linked report. Some
provider adapters may need work; no hosted or enterprise readiness implied.

**Beta candidate:** Contributors have confirmed live data flows across the supported providers where permitted, the
source contracts and deletion behavior have automated tests, and at least five independent testers finish the quick
start and provide feedback.

**Stable 1.0 candidate:** Complete security/privacy review, accessible UI, maintainable connector contracts, stable data
migrations, clear compatibility policy, repeatable release process and independent installation feedback.

Use [GitHub Issues](https://github.com/savisaluwadana/Pain-Point-Analyzer/issues) to propose or claim tasks. We
especially welcome documentation, tests and narrow connector improvements.
