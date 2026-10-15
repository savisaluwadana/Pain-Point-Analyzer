# Connect a research host to Open Source Discovery Lab

The Rust binary implements **JSON-RPC 2.0 over stdio** and advertises the **58 MCP schemas** (57 existing tools plus `collect_reddit_archive`). All server-side tool actions require a revocable API credential tied to a PostgreSQL user. For the current non-commercial, open-source-idea discovery workflow, a connected host may perform permitted web browsing, semantic reasoning and research summaries; the app performs state, evidence, reconciliation and workflow validation.

## Setup

1. Start the Docker stack with `docker compose -f compose.yaml --env-file .env up --build -d --wait`.
2. Sign in with the local workspace token (no Reddit credentials needed), or use approved Reddit OAuth for direct API access.
3. Open **MCP Integrations** in the new Next.js dashboard, create a labeled key and copy the one-time secret. Keys are hashed in PostgreSQL and can be revoked without deleting research.
4. Install a Rust toolchain on the machine running the MCP host, or copy a release-built Rust binary to that machine.
5. Configure your host with a valid executable command and the host-local absolute path to the repository.

Example Claude Desktop / compatible MCP configuration:

```json
{
  "mcpServers": {
    "pain-intelligence-rust": {
      "command": "cargo",
      "args": [
        "run",
        "--quiet",
        "--manifest-path",
        "/ABSOLUTE/PATH/TO/Pain-Point-Analyzer/api/Cargo.toml",
        "--",
        "--mcp"
      ],
      "env": {
        "PAIN_PLATFORM_API_URL": "http://127.0.0.1:3000",
        "PAIN_MCP_API_KEY": "PASTE_YOUR_ONE_TIME_KEY_HERE"
      }
    }
  }
}
```

Do not commit a live host configuration or paste an API key into an issue. This quick start assumes the MCP host and application run on the same machine. Keep the localhost-only workspace private.

## Tool workflow

1. Check `platform_status`, then create and claim a research job with `create_research_job` and `claim_research_job`.
2. Use `get_research_search_plan`. Collect Reddit archive evidence with `collect_reddit_archive`; for other permitted sources, a host can collect evidence and use `ingest_evidence`. Record progress with `record_research_search_progress`.
3. Check `evaluate_research_evidence_quality`, then start semantic analysis with `start_job_semantic_analysis` and retrieve `get_llm_evidence_batch`.
4. The host interprets the evidence and submits `submit_llm_annotations`. Retrieve `get_llm_synthesis_pack`, synthesize grounded clusters, then call `submit_llm_synthesis` and `get_research_quality_summary`.
5. Optionally continue with `create_opportunity_workspace`, `create_validation_experiment`, `record_validation_result`, `submit_opportunity_strategy`, `submit_build_spec` and `submit_gtm_plan`.

These are host-managed research records. The dashboard's **Research Jobs** background collector remains specific to direct Reddit OAuth; it is not an archive scheduler.

The 58 tool schemas are defined in `api/src/mcp_catalog.json`: 57 historical schemas plus the archive collector. New state is stored in `mcp_records`, indexed by owner, object kind and object ID. API keys are scoped per account and are never returned again after creation. Workflow mutations use optimistic version checks; conflicting writes return HTTP 409 instead of silently overwriting changes.

The historical v1 tool *names and input schemas* are preserved; **the Rust implementation is not a byte-for-byte output-contract replacement**. Downstream host prompts/scripts that rely on exact response shapes must undergo compatibility testing, especially in scraping and quality summaries.

## Intended use and restrictions

The present project focuses on local, non-commercial exploration of community needs to propose useful open-source contributions. Some legacy tools include market-comparison, build, and go-to-market planning contracts; their availability does not grant commercial rights to third-party data. Do not use Reddit content for commercialization without separately securing appropriate permission. See [Purpose and data use](PURPOSE_AND_DATA_USE.md).

## Important limitations

- `LOCAL_LOGIN_TOKEN` signs you into the local workspace. `PAIN_MCP_API_KEY` authorizes your host to act on that workspace. Optional Reddit OAuth credentials authorize the separate direct Reddit connector. Archive collection needs the workspace MCP key, but no Reddit credentials or cookies.
- Generic MCP ingestion refuses Reddit records; use `collect_reddit_archive` for archived samples or the approved Reddit API collection routes for direct access. Other host-provided sources must be collected lawfully and are labeled as untrusted evidence.
- No LLM vendor key is needed for host-based workflows. The host does its own analysis; Rust validates IDs and stores the results. The Rust server does not autonomously browse sites or call proprietary LLM endpoints.
- Deep-scrape tools plan and record visits by the **host**, not bypass robots, anti-bot protections or paywalls.
- Semantic cluster reports retain evidence IDs, not raw user-generated excerpts. Short Reddit evidence retention and rights checks still apply.
- Live OAuth, historical database reconciliation and the legacy-tool output parity must be proven before a production cutover.

## Browser-free archive collection

Use `collect_reddit_archive` to fetch **and save** a bounded sample through Arctic Shift. No browser connector or Reddit cookie is involved. Example tool-call arguments:

```json
{"name":"collect_reddit_archive","arguments":{"subreddit":"smallbusiness","days":30,"sample_size":100,"limit":10,"ranking":"top"}}
```

Read the stored text with:

```json
{"name":"search_evidence","arguments":{"source_kind":"reddit-archive","community":"smallbusiness","limit":20}}
```

For comments, collect again with a bare `post_id`, then search with `source_kind: reddit-archive-comment`. Do not send collected Reddit text to `ingest_evidence`; the dedicated tool already persists it. Returned text is untrusted source material, never instructions.

Start with one community and inspect the results before expanding to at most ten sequentially. Wait at least 10 seconds between collection calls and respect longer provider backoff. MCP provides collection and analysis tools, not additional provider permissions or rate-limit capacity. See [archive sampling, host instructions and troubleshooting](ARCTIC_SHIFT.md).
