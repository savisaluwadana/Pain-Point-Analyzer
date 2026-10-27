# HTTP API reference

This page documents the **implemented Rust/Axum HTTP routes** on the local
application, as registered in
[`api/src/main.rs`](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/api/src/main.rs).
For the authoritative request schemas and limits, follow the linked Rust files.
The application does not publish an OpenAPI specification today.

## Base URL and authentication

- Local browser/API origin: `http://localhost:3000/api` (Next.js proxies to Rust).
- Native Rust listens on port 4000 inside the Compose network, not published by default.
- Browser endpoints use an HttpOnly `pil_session` cookie obtained after
  local login or approved Reddit OAuth.
- State-changing browser requests require a matching `Origin` header
  (`http://localhost:3000` in the default setup).
- `/api/mcp/execute` uses a revocable `Authorization: Bearer ...` key,
  **not** the browser session.
- Errors are JSON with `{"error":"message"}` and a relevant HTTP status.
  `401` means missing/invalid credentials, `403` origin rejected,
  `404` unavailable or not owned, `429` rate/concurrency budget, and
  `502/503` provider or configuration failures.

!!! warning "Local API, not a public hosted service"
    The local-token alpha does not support publicly exposing API routes, a
    multi-tenant frontend, or production-grade identity management.

## Routes by domain

| Method | Route | Purpose |
| --- | --- | --- |
| GET | `/api/health` | Database-backed health status; no login |
| POST | `/api/auth/local` | Local token sign-in, `{"token":"…"}`; needs same-origin |
| GET | `/api/auth/reddit/start` | Begin approved OAuth; disabled by default |
| GET | `/api/auth/reddit/callback` | OAuth callback |
| GET | `/api/me` | Current authenticated workspace identity |
| POST | `/api/auth/logout` | Invalidate current browser session |
| POST | `/api/auth/reddit/disconnect` | Remove local Reddit link and attempt provider revocation |
| GET | `/api/sources` | Available collector catalog and setup status |
| POST | `/api/sources/collect` | Collect bounded source-linked evidence |
| GET / POST | `/api/projects` | List/create research projects |
| DELETE | `/api/projects/{id}` | Delete a project; other evidence is retained as unassigned |
| GET / POST | `/api/evidence` | List or ingest manually permitted evidence |
| DELETE | `/api/evidence/{id}` | Delete one owned evidence item |
| POST | `/api/research/analyze` | Run deterministic topic analysis and save findings |
| GET | `/api/research/runs` | Recent saved analysis reports |
| POST | `/api/research/scan` | Approved Reddit post sample |
| POST | `/api/research/thread` | Approved Reddit thread comments |
| GET / POST | `/api/research/jobs` | List/queue direct OAuth Reddit worker jobs |
| GET | `/api/opportunities` | List candidates and human review fields |
| GET | `/api/opportunities/{id}/brief` | Hydrate owner-scoped, hash-checked representative source excerpts |
| PUT | `/api/opportunities/{id}/review` | Save a manual decision and validation notes |
| GET | `/api/mcp/keys` | List revocable key metadata |
| POST | `/api/mcp/keys` | Create a per-owner MCP credential; one-time secret |
| DELETE | `/api/mcp/keys/{id}` | Revoke an MCP key |
| POST | `/api/mcp/execute` | Execute an authorized MCP tool by name |
| GET | `/api/mcp/runs` | List current owner's host-provided research runs |
| GET | `/api/mcp/workspaces` | List current owner's host-provided workspaces |
| GET | `/api/mcp/research-jobs` | List current owner's host research jobs |
| GET | `/api/privacy/export` | Download bounded account export without credentials or Reddit body text |
| POST | `/api/privacy/delete-data` | Delete owner data and sessions; attempts remote token revocation |

### Local login example

Use the browser for normal operation. For local API testing:

```bash
read -rsp 'Local login token: ' LOCAL_TOKEN; echo
curl --fail-with-body -c /tmp/pain-session.txt \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  -d "$(printf '{"token":"%s"}' "$LOCAL_TOKEN")" \
  http://localhost:3000/api/auth/local
unset LOCAL_TOKEN
curl --fail-with-body -b /tmp/pain-session.txt http://localhost:3000/api/me
```

Never commit cookie files or put a real token directly in a committed script.
Use a shell with private access and delete `/tmp/pain-session.txt` after testing.

## Source collection

```http
POST /api/sources/collect
Content-Type: application/json

{"source":"github","query":"missing integration","project_id":null,"limit":15}
```

Supported source identifiers: `arctic-shift`, `github`, `hackernews`,
`stackoverflow`, `bluesky`, `youtube`, `discourse`, `medium`.
Non-archive connectors collect **1–25 records per request** and share a
per-owner, per-source budget of four requests/minute. Additional source
limits and provider quotas apply.

For the archive, use a **bare subreddit** as `target`:

```json
{
  "source": "arctic-shift",
  "target": "smallbusiness",
  "query": "",
  "days": 30,
  "ranking": "top",
  "sample_size": 100,
  "limit": 10
}
```

Archive collection is **on-demand** and uses a separate provider-wide
backoff. Results are not complete live Reddit rankings. See
[archive parameters and restrictions](ARCTIC_SHIFT.md).

## Projects, evidence, analysis

Create a project:

```json
{"name":"Developer workflow research","description":"Investigate manual handoffs"}
```

Manually add non-Reddit evidence through `POST /api/evidence`:

```json
{
  "source_kind": "manual",
  "source_id": "my-observation-001",
  "source_url": "https://example.com/fictional-report",
  "title": "Fictional reporting handoff",
  "body": "A fictional operator manually reconciles two reports.",
  "project_id": null
}
```

Only `manual` or `import` source kinds are allowed on this endpoint;
Reddit content must use its authorized path. The project reference, if
supplied, must belong to the signed-in user.

Run an analysis:

```json
{
  "topic": "Recurring developer workflow bottlenecks",
  "project_id": null,
  "limit": 1000
}
```

The analyzer allows limits up to 10,000 and returns a `run_id` plus a
versioned `report`, containing clusters, coverage, independent thread counts
and relative priorities. A stored report contains references and hashes
rather than durable copied evidence excerpts. Representative quotes are
rehydrated on reads, and may become unavailable.

## Opportunity review

Retrieve a brief with
`GET /api/opportunities/{id}/brief`. Its `cluster` may be `null` for
legacy or orphaned reports. This is not a complete evidence export.

Save a manual review through `PUT /api/opportunities/{id}/review`:

```json
{
  "decision": "needs_validation",
  "validation_notes": "Interview two independent maintainers and record contradicting findings.",
  "existing_solutions": "Existing OSS tools and links from my own comparison."
}
```

Allowed stages: `unreviewed`, `investigating`, `needs_validation`,
`ready_to_build`, `not_pursuing`. Notes are limited to 4,000 characters
and solutions to 2,000 characters. The ready-to-build stage requires
at least 20 characters of validation notes; it is explicitly a human
choice, not an externally verified milestone.

## MCP request and response

```http
POST /api/mcp/execute
Authorization: Bearer <your-revocable-MCP-key>
Content-Type: application/json

{"name":"platform_status","arguments":{}}
```

The service validates the tool name and required arguments against
[`api/src/mcp_catalog.json`](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/api/src/mcp_catalog.json).
The bridge communicates JSON-RPC 2.0 over stdio and calls the same HTTP
endpoint. The tool result schema depends on its tool definition;
[see the tool index](MCP_TOOL_INDEX.md).

## Privacy, consistency and limits

List endpoints are bounded; they are not unbounded cursor APIs.
Projects, evidence, reports and opportunity reviews are owner-scoped.
Deleting source evidence does **not** rewrite saved reports, but on-read
quote hydration checks the source's current hash. Reddit content is not
included as raw text in the personal export. See
[data model](DATA_MODEL.md), [security](SUPPORT_AND_SECURITY.md), and
[operations](OPERATIONS.md).
