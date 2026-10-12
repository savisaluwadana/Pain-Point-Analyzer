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
call claim_research_job '{}' | jq -e '.job == null' >/dev/null
call start_scrape_session "{\"job_id\":\"$job\"}" | jq -e '.session.status == "running"' >/dev/null
call add_scrape_candidates "{\"job_id\":\"$job\",\"candidates\":[{\"url\":\"https://example.com/review-one\"}]}" | jq -e '.session.candidates | length == 1' >/dev/null
call get_next_scrape_batch "{\"job_id\":\"$job\",\"limit\":1}" | jq -e '.batch | length == 1' >/dev/null
call record_scrape_page_result "{\"job_id\":\"$job\",\"url\":\"https://example.com/review-one\",\"evidence_added\":1}" | jq -e '.session.pages | length == 1' >/dev/null
call get_scrape_session_summary "{\"job_id\":\"$job\"}" | jq -e '.pages | length == 1' >/dev/null
call get_research_quality_summary "{\"run_id\":\"$run\"}" | jq -e '.clusters == 1' >/dev/null
call get_market_sizing_pack "{\"run_id\":\"$run\"}" | jq -e '.opportunities | length == 1' >/dev/null
call submit_market_sizing_assessment "{\"run_id\":\"$run\",\"opportunity_id\":\"reporting-saas\",\"sources\":[{\"url\":\"https://example.com/pricing\"}]}" | jq -e '.sizing.opportunity_id == "reporting-saas"' >/dev/null
call submit_opportunity_strategy "{\"workspace_id\":\"$workspace\",\"icp\":\"Small operations teams\",\"wedge\":\"Faster recurring reports\"}" | jq -e '.workspace.strategy.icp == "Small operations teams"' >/dev/null
experiment="$(call create_validation_experiment "{\"workspace_id\":\"$workspace\",\"type\":\"customer-interviews\",\"hypothesis\":\"Teams need it\"}" | jq -r '.experiment_id')"
call record_validation_result "{\"workspace_id\":\"$workspace\",\"experiment_id\":\"$experiment\",\"verdict\":\"supports\",\"status\":\"complete\"}" | jq -e '.workspace.experiments[0].verdict == "supports"' >/dev/null
call submit_build_spec "{\"workspace_id\":\"$workspace\",\"product_name\":\"Workflow Report\",\"one_liner\":\"Eliminate reporting drudgery\"}" | jq -e '.workspace.build_spec.product_name == "Workflow Report"' >/dev/null
call submit_gtm_plan "{\"workspace_id\":\"$workspace\",\"offer\":\"Design partner pilot\"}" | jq -e '.workspace.gtm_plan.offer == "Design partner pilot"' >/dev/null
call set_opportunity_stage "{\"workspace_id\":\"$workspace\",\"stage\":\"building\"}" | jq -e '.workspace.stage == "building"' >/dev/null
printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | docker compose exec -T -e "PAIN_MCP_API_KEY=$token" -e PAIN_PLATFORM_API_URL=http://web:3000 api /usr/local/bin/pain-intelligence-api --mcp | jq -e '.result.tools | length == 57' >/dev/null
# Browser session mutations require an exact same-origin Origin header.
session='local_integration_cookie_do_not_use'
session_hash="$(printf '%s' "$session" | sha256sum | cut -c1-64)"
sql "INSERT INTO user_sessions(token_hash,user_id,expires_at) VALUES('$session_hash','$owner',now()+interval '1 day')" >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/projects" \
  -H "Cookie: pil_session=$session" -H 'Origin: https://evil.example' \
  -H 'Content-Type: application/json' --data '{"name":"blocked project"}')" = 403
test "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/projects" \
  -H "Cookie: pil_session=$session" \
  -H 'Content-Type: application/json' --data '{"name":"missing origin"}')" = 403
curl --fail-with-body -sS -X POST "$api/projects" -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  --data '{"name":"Permitted research project"}' | jq -e '.name == "Permitted research project"' >/dev/null
test "$(sql "SELECT count(*) FROM information_schema.tables WHERE table_name='reddit_api_budgets'")" = 1
# Deleting a project reassigns its evidence (rather than deleting unrelated research).
project_id="$(curl --fail-with-body -sS "$api/projects" -H "Cookie: pil_session=$session" | jq -r '.[0].id')"
curl --fail-with-body -sS "$api/projects/$project_id" -X DELETE \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' | jq -e '.deleted == true' >/dev/null
curl --fail-with-body -sS "$api/privacy/export" -H "Cookie: pil_session=$session" | \
  jq -e '.account.reddit_username == "ci-researcher" and (.projects|length) == 0 and (.host_records|length) >= 2 and .truncated == false' >/dev/null
# Revocable API credentials are shown only once; revoked credentials cannot call tools.
issued="$(curl --fail-with-body -sS -X POST "$api/mcp/keys" \
 -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
 -H 'Content-Type: application/json' --data '{"label":"CI temporary MCP client"}')"
issued_secret="$(printf '%s' "$issued" | jq -r '.api_key')"
issued_id="$(printf '%s' "$issued" | jq -r '.id')"
[[ "$issued_secret" == pil_* ]]
curl --fail-with-body -sS "$api/mcp/keys" -H "Cookie: pil_session=$session" | \
 jq -e '.[] | select(.label=="CI temporary MCP client") | .id' >/dev/null
curl --fail-with-body -sS -X DELETE "$api/mcp/keys/$issued_id" -H "Cookie: pil_session=$session" \
 -H 'Origin: http://localhost:3000' | jq -e '.revoked == true' >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/mcp/execute" \
 -H "Authorization: Bearer $issued_secret" -H 'Content-Type: application/json' \
 --data '{"name":"platform_status","arguments":{}}')" = 401
other="$(sql "INSERT INTO users(reddit_id,reddit_username) VALUES('ci-other-id','ci-other') ON CONFLICT (reddit_id) DO UPDATE SET reddit_username=excluded.reddit_username RETURNING id")"
other_token='pil_other_user_key'
other_hash="$(printf '%s' "$other_token" | sha256sum | cut -c1-64)"
sql "INSERT INTO mcp_api_keys(owner_id,key_hash,label) VALUES('$other','$other_hash','Second account')" >/dev/null
curl --fail-with-body -sS "$api/mcp/execute" -H "Authorization: Bearer $other_token" -H 'Content-Type: application/json' --data '{"name":"search_evidence","arguments":{"q":"spreadsheets"}}' | jq -e '.returned == 0' >/dev/null
# Corrupted or expired stored credentials must not block local source-data deletion.
sql "INSERT INTO reddit_credentials(user_id,access_token_ciphertext,expires_at) VALUES('$owner','invalid-local-test-token',now()+interval '1 hour')" >/dev/null
curl --fail-with-body -sS "$api/auth/reddit/disconnect" -X POST -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' | jq -e '.disconnected == true and .provider_token_revoked == false' >/dev/null
test "$(sql "SELECT count(*) FROM reddit_credentials WHERE user_id='$owner'")" = 0
curl --fail-with-body -sS "$api/me" -H "Cookie: pil_session=$session" | \
  jq -e '.reddit_connected == false' >/dev/null
evidence_id="$(call search_evidence '{"q":"spreadsheets"}' | jq -r '.items[0].id')"
curl --fail-with-body -sS "$api/evidence/$evidence_id" -X DELETE \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' | jq -e '.deleted == true' >/dev/null
call search_evidence '{"q":"spreadsheets"}' | jq -e '.returned == 0' >/dev/null
curl --fail-with-body -sS "$api/privacy/delete-data" -X POST \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' | jq -e '.deleted == true' >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/me" -H "Cookie: pil_session=$session")" = 401
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/mcp/execute" \
  -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
  --data '{"name":"platform_status","arguments":{}}')" = 401
echo "Rust MCP integration: authorized workflows, 57 schemas, and tenant isolation passed."
