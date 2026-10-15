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
        "/ABSOLUTE/PATH/TO/Reddit-post-Analyzer/api/Cargo.toml",
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

Do not commit a live host configuration or paste an API key into an issue. When the host runs on another machine, make the web endpoint reachable only over authorized HTTPS and configure the appropriate URL.

## Tool workflow

```text
platform_status
  -> create_research_job
  -> claim_research_job
  -> get_research_search_plan
  -> [host visits permitted external sources]
  -> ingest_evidence
  -> record_research_search_progress
  -> evaluate_research_evidence_quality
  -> start_job_semantic_analysis
  -> get_llm_evidence_batch
  -> [host semantically interprets evidence]
  -> submit_llm_annotations
  -> get_llm_synthesis_pack
  -> [host synthesizes grounded clusters and candidates]
  -> submit_llm_synthesis
  -> get_research_quality_summary
  -> create_opportunity_workspace
  -> create_validation_experiment
  -> record_validation_result
  -> submit_opportunity_strategy
  -> submit_build_spec
  -> submit_gtm_plan
```

All tool schemas are copied directly from the v1 generated reference in `api/src/mcp_catalog.json`. New state is stored in `mcp_records`, indexed by owner, object kind and object ID. API keys are scoped per account and are never returned again after creation. Workflow mutations use optimistic version checks; conflicting writes return HTTP 409 instead of silently overwriting changes.

The historical v1 tool *names and input schemas* are preserved; **the Rust implementation is not a byte-for-byte output-contract replacement**. Downstream host prompts/scripts that rely on exact response shapes must undergo compatibility testing, especially in scraping and quality summaries.

## Intended use and restrictions

The present project focuses on local, non-commercial exploration of community needs to propose useful open-source contributions. Some legacy tools include market-comparison, build, and go-to-market planning contracts; their availability does not grant commercial rights to third-party data. Do not use Reddit content for commercialization without separately securing appropriate permission. See [Purpose and data use](PURPOSE_AND_DATA_USE.md).

## Important limitations

- The OAuth **user identity** and the **MCP API key** are different credentials. The former authorizes only approved Reddit API access; the latter authorizes the owner's database research tools. You cannot use a raw Reddit password or browser session.
- Generic MCP ingestion refuses Reddit records; use `collect_reddit_archive` for archived samples or the approved Reddit API collection routes for direct access. Other host-provided sources must be collected lawfully and are labeled as untrusted evidence.
- No LLM vendor key is needed for host-based workflows. The host does its own analysis; Rust validates IDs and stores the results. The Rust server does not autonomously browse sites or call proprietary LLM endpoints.
- Deep-scrape tools plan and record visits by the **host**, not bypass robots, anti-bot protections or paywalls.
- Semantic cluster reports retain evidence IDs, not raw user-generated excerpts. Short Reddit evidence retention and rights checks still apply.
- Live OAuth, historical database reconciliation and the legacy-tool output parity must be proven before a production cutover.

## Browser-free archive collection

Use `collect_reddit_archive` to fetch and store bounded samples through Arctic Shift. No browser or Reddit cookie is involved. See [archive setup and examples](ARCTIC_SHIFT.md).
