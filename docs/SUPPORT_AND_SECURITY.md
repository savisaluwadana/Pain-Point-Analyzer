# Support, privacy and security

## Project maturity

Open Source Discovery Lab is an **early community alpha**. The code is
MIT-licensed, but the intended research operation is currently local,
individual and non-commercial. A public documentation site must not be
mistaken for a hosted version of the application.

The maintainers do not promise service uptime, provider access, security
certification or a support response-time guarantee.

## Get help and contribute

- [Open a support or bug issue](https://github.com/savisaluwadana/Pain-Point-Analyzer/issues/new/choose)
- [Read repository support policy](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/SUPPORT.md)
- [Contribute changes](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/CONTRIBUTING.md)
- [Check the roadmap](ROADMAP.md) and [development guide](DEVELOPMENT.md)

For troubleshooting, redact cookies, bearer tokens, identifying details,
raw third-party posts and private research notes. Use fictional test data
when sharing a reproduction.

## Report security vulnerabilities privately

**Do not post vulnerability details or exploitable proofs in public issues.**
Follow the repository [security policy](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/SECURITY.md),
including its preferred private reporting channel. Check whether GitHub
**Report a vulnerability** is available in the repository Security tab.
Do not assume an SLA or public production-hardening claim.

## Authentication and access boundaries

- Local browser sign-in uses a high-entropy `LOCAL_LOGIN_TOKEN` only
  when the app is configured with a localhost `WEB_URL`.
- Browser sessions are cookie-based; mutations require same-origin
  checks, and cookies are HttpOnly.
- MCP tools use revocable bearer keys tied to the research account.
- Owned evidence and opportunity reviews are scoped by account ID in
  database queries. Session/owner isolation is covered by tests, but
  this is not an independent full security audit.
- External provider secrets are not part of a public static Pages build.
  Do not put secrets into Markdown, screenshots or workflow variables.
- Provider requests enforce host, content-size and timeout restrictions,
  and should distinguish permission denial from rate limiting.

## What remains unverified

The project has not completed a production security review, full
manual accessibility audit, broad live-connector acceptance suite or
provider authorization for commercial Reddit data use. Existing
`keywords-v2` heuristics and optional host-supplied AI results should
not be advertised as validated predictions.

## Data use and personal information

Third-party source material may be copyrighted, sensitive, deleted, or
subject to provider terms. The software's MIT license does not transfer
rights in that material. Respect the user's rights and source rules;
avoid scraping private content, automated outreach, profiling,
redistribution or treating cached archives as an authorization.

[Purpose and data-use policy](PURPOSE_AND_DATA_USE.md) ·
[Evidence lifecycle and deletion](DATA_MODEL.md) ·
[Local operations](OPERATIONS.md)
