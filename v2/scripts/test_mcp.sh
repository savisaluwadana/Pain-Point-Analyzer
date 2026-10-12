#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
api="http://127.0.0.1:3000/api"
sql() { docker compose exec -T postgres psql -q -v ON_ERROR_STOP=1 -U pain -d pain_intelligence -Atc "$1"; }
token='pil_integration_mock_key_only_for_ci'
owner="$(sql "INSERT INTO users(reddit_id,reddit_username) VALUES('ci-owner-id','ci-researcher') ON CONFLICT (reddit_id) DO UPDATE SET reddit_username=excluded.reddit_username RETURNING id")"
hash="$(printf '%s' "$token" | sha256sum | cut -c1-64)"
sql "INSERT INTO mcp_api_keys(owner_id,key_hash,label) VALUES('$owner','$hash','CI integration key')" >/dev/null
call() {
  local name="$1" args="$2"
  curl --fail-with-body -sS "$api/mcp/execute" -H "Authorization: Bearer $token" -H "Content-Type: application/json" --data "{\"name\":\"$name\",\"arguments\":$args}"
}
test "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/mcp/execute" -H 'Content-Type: application/json' --data '{"name":"platform_status","arguments":{}}')" = 401
call platform_status '{}' | jq -e '.ok == true and .schemaTools == 57' >/dev/null
call ingest_evidence '{"items":[{"source_kind":"forum","external_id":"ci-one","source_name":"CI Forum","url":"https://example.com/one","title":"Manual reporting","text":"Manual spreadsheets slow down the team every week."}]}' | jq -e '.ingested == 1' >/dev/null
call search_evidence '{"q":"spreadsheets"}' | jq -e '.returned == 1' >/dev/null
call source_stats '{}' | jq -e '.total == 1' >/dev/null
call analyze_pain_points '{"save_as":"integration scan"}' | jq -e '.report.evidence_scanned == 1' >/dev/null
job="$(call create_research_job '{"topic":"Manual reporting bottlenecks"}' | jq -r '.job.job_id')"
call get_research_search_plan "{\"job_id\":\"$job\"}" | jq -e '.missions | length >= 4' >/dev/null
call record_research_search_progress "{\"job_id\":\"$job\",\"queries\":[{\"query\":\"manual reporting workflows\"}]}" | jq -e '.memory.queries | length == 1' >/dev/null
call claim_research_job '{}' | jq -e '.job.status == "claimed"' >/dev/null
run="$(call start_job_semantic_analysis "{\"job_id\":\"$job\"}" | jq -r '.run_id')"
evidence_id="$(call get_llm_evidence_batch "{\"run_id\":\"$run\"}" | jq -r '.evidence[0].id')"
test -n "$evidence_id"
call submit_llm_annotations "{\"run_id\":\"$run\",\"annotations\":[{\"evidence_id\":\"$evidence_id\",\"canonical_pain\":\"Manual reporting takes hours\",\"semantic_cluster_key\":\"manual-reports\",\"semantic_cluster_label\":\"Manual reporting\"}]}" | jq -e '.annotated == 1' >/dev/null
call submit_llm_synthesis "{\"run_id\":\"$run\",\"clusters\":[{\"cluster_id\":\"manual-reports\",\"label\":\"Manual reporting\",\"problem_statement\":\"Workaround burden\",\"evidence_ids\":[\"$evidence_id\"]}],\"opportunities\":[{\"opportunity_id\":\"reporting-saas\",\"title\":\"Reporting workflow automation\",\"problem\":\"Manual work\",\"cluster_ids\":[\"manual-reports\"],\"evidence_ids\":[\"$evidence_id\"],\"opportunity_score\":30}]}" | jq -e '.run.status == "synthesized"' >/dev/null
workspace="$(call create_opportunity_workspace "{\"run_id\":\"$run\",\"opportunity_id\":\"reporting-saas\"}" | jq -r '.workspace.workspace_id')"
call get_opportunity_execution_pack "{\"workspace_id\":\"$workspace\"}" | jq -e '.workspace.stage == "discovery"' >/dev/null
printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | docker compose exec -T -e "PAIN_MCP_API_KEY=$token" -e PAIN_PLATFORM_API_URL=http://web:3000 api /usr/local/bin/pain-intelligence-api --mcp | jq -e '.result.tools | length == 57' >/dev/null
other="$(sql "INSERT INTO users(reddit_id,reddit_username) VALUES('ci-other-id','ci-other') ON CONFLICT (reddit_id) DO UPDATE SET reddit_username=excluded.reddit_username RETURNING id")"
other_token='pil_other_user_key'
other_hash="$(printf '%s' "$other_token" | sha256sum | cut -c1-64)"
sql "INSERT INTO mcp_api_keys(owner_id,key_hash,label) VALUES('$other','$other_hash','Second account')" >/dev/null
curl --fail-with-body -sS "$api/mcp/execute" -H "Authorization: Bearer $other_token" -H 'Content-Type: application/json' --data '{"name":"search_evidence","arguments":{"q":"spreadsheets"}}' | jq -e '.returned == 0' >/dev/null
echo "Rust MCP integration: authorized workflows, 57 schemas, and tenant isolation passed."
