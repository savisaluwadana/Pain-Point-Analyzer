#!/usr/bin/env bash
set -Eeuo pipefail
trap 'echo "MCP integration failed at line $LINENO" >&2' ERR
cd "$(dirname "$0")/.."
api="http://127.0.0.1:3000/api"
sql() { docker compose exec -T postgres psql -q -v ON_ERROR_STOP=1 -U pain -d pain_intelligence -Atc "$1"; }
token='pil_integration_mock_key_only_for_ci'
owner="$(sql "INSERT INTO users(reddit_id,reddit_username) VALUES('ci-owner-id','ci-researcher') ON \
CONFLICT (reddit_id) DO UPDATE SET reddit_username=excluded.reddit_username RETURNING id")"
hash="$(printf '%s' "$token" | sha256sum | cut -c1-64)"
sql "INSERT INTO mcp_api_keys(owner_id,key_hash,label) VALUES('$owner','$hash','CI integration key')" >/dev/null
call() {
  local name="$1" args="$2"
  curl --fail-with-body -sS "$api/mcp/execute" -H "Authorization: Bearer $token" \
    -H "Content-Type: application/json" --data "{\"name\":\"$name\",\"arguments\":$args}"
}
test \
  "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/mcp/execute" \
    -H 'Content-Type: application/json' --data '{"name":"platform_status","arguments":{}}')" \
  = 401
call platform_status '{}' | jq -e '.ok == true and .schemaTools == 58' >/dev/null
call ingest_evidence '{
  "items": [
    {
      "source_kind": "forum",
      "external_id": "ci-one",
      "source_name": "CI Forum",
      "url": "https://example.com/one",
      "title": "Manual reporting",
      "text": "Manual spreadsheets slow down the team every week."
    }
  ]
}' | jq -e '.ingested == 1' >/dev/null
call search_evidence '{"q":"spreadsheets"}' | jq -e '.returned == 1' >/dev/null
call source_stats '{}' | jq -e '.total == 1' >/dev/null
# Archive connector is available without Reddit credentials. Invalid targets must fail
# before any upstream request or budget reservation; CI never calls the live archive.
curl --fail-with-body -sS "$api/sources" | jq -e '.sources[] | select(.id == "arctic-shift" and .ready == true)' >/dev/null
test \
  "$(curl -s -o /dev/null -w '%{http_code}' "$api/mcp/execute" \
    -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
    --data '{"name":"collect_reddit_archive","arguments":{"subreddit":"../invalid"}}')" \
  = 400
test "$(sql "SELECT count(*) FROM archive_provider_budget")" = 0

# Re-importing a source preserves its ID and updates its canonical link.
call ingest_evidence '{
  "items": [
    {
      "source_kind": "forum",
      "external_id": "ci-one",
      "source_name": "CI Forum",
      "url": "https://example.com/updated-source",
      "title": "Manual reporting",
      "text": "Manual spreadsheets slow down the team every week."
    }
  ]
}' | jq -e '.ingested == 1' >/dev/null
call search_evidence '{"q":"spreadsheets"}' | jq -e \
  '.returned == 1 and .items[0].url == "https://example.com/updated-source"' >/dev/null
call analyze_pain_points '{"save_as":"integration scan"}' | jq -e '.report.evidence_scanned == 1' >/dev/null
job="$(call create_research_job '{"topic":"Manual reporting bottlenecks"}' | jq -r '.job.job_id')"
call get_research_search_plan "{\"job_id\":\"$job\"}" | jq -e '.missions | length >= 4' >/dev/null
call record_research_search_progress "{\"job_id\":\"$job\",\"queries\":[{\"query\":\"manual reporting workflows\"}]}" | jq \
  -e '.memory.queries | length == 1' >/dev/null
# Existing unrelated evidence must never make a fresh job appear ready.
call evaluate_research_job_coverage "{\"job_id\":\"$job\"}" | jq \
  -e '.evidence_total == 0 and .ready_for_synthesis == false' >/dev/null
call ingest_evidence "{\"batch_id\":\"$job\",\"items\":[{\"source_kind\":\"forum\",\"external_id\":\"ci-linked-job\",\
\"url\":\"https://example.com/job\",\"text\":\"Repeated manual reporting takes hours\"}]}" | jq \
  -e '.ingested == 1' >/dev/null
call evaluate_research_job_coverage "{\"job_id\":\"$job\"}" | jq \
  -e '.evidence_total == 1 and .ready_for_synthesis == false' >/dev/null
# Re-upserting existing evidence into a different batch must NOT remove the old batch link.
other_batch="$(call create_research_job '{"topic":"Secondary research batch"}' | jq -r '.job.job_id')"
call ingest_evidence \
  "{\"batch_id\":\"$other_batch\",\"items\":[{\"source_kind\":\"forum\",\"external_id\":\"ci-linked-job\",\
\"url\":\"https://example.com/job\",\"text\":\"Repeated manual reporting takes hours\"}]}" | jq \
  -e '.ingested == 1' >/dev/null
call evaluate_research_job_coverage "{\"job_id\":\"$job\"}" | jq -e '.evidence_total == 1' >/dev/null
call evaluate_research_job_coverage "{\"job_id\":\"$other_batch\"}" | jq -e '.evidence_total == 1' >/dev/null
call fail_research_job "{\"job_id\":\"$other_batch\",\"reason\":\"CI fixture completed\"}" | jq \
  -e '.job.status == "failed"' >/dev/null
# New research job is intentionally queued after the original: claim oldest first.
claim="$(call claim_research_job '{}')"
printf '%s' "$claim" | jq -e '.job.status == "claimed" and (.job.claim_token|length) > 10' >/dev/null
claim_token="$(printf '%s' "$claim" | jq -r '.job.claim_token')"
call get_research_job "{\"job_id\":\"$job\"}" | jq -e '.job.claim_token == null' >/dev/null
call list_research_jobs '{}' | jq -e 'all(.jobs[]; .claim_token == null)' >/dev/null
test \
  "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/mcp/execute" \
    -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
    --data "{\"name\":\"start_job_semantic_analysis\",\"arguments\":{\"job_id\":\"$job\",\"claim_token\":\"wrong\"}}")" \
  = 400
test \
  "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/mcp/execute" \
    -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
    --data "{\"name\":\"heartbeat_research_job\",\"arguments\":{\"job_id\":\"$job\",\"claim_token\":\"wrong\"}}")" \
  = 400
call heartbeat_research_job "{\"job_id\":\"$job\",\"claim_token\":\"$claim_token\",\"status\":\"collecting\"}" | jq \
  -e '.job.status == "collecting"' >/dev/null
run="$(call start_job_semantic_analysis "{\"job_id\":\"$job\",\"claim_token\":\"$claim_token\"}" | jq -r '.run_id')"
evidence_id="$(call get_llm_evidence_batch "{\"run_id\":\"$run\"}" | jq -r '.evidence[0].id')"
test -n "$evidence_id"
call submit_llm_annotations \
  "{\"run_id\":\"$run\",\"annotations\":[{\"evidence_id\":\"$evidence_id\",\"canonical_pain\":\"Manual \
reporting takes hours\",\
\"semantic_cluster_key\":\"manual-reports\",\"semantic_cluster_label\":\"Manual reporting\"}]}" | jq \
  -e '.annotated == 1' >/dev/null
call submit_llm_synthesis \
  "{\"run_id\":\"$run\",\"clusters\":[{\"cluster_id\":\"manual-reports\",\"label\":\"Manual reporting\",\
\"problem_statement\":\"Workaround burden\",\"evidence_ids\":[\"$evidence_id\"]}],\
\"opportunities\":[{\"opportunity_id\":\"reporting-saas\",\"title\":\"Reporting \
workflow automation\",\"problem\":\"Manual work\",\"cluster_ids\":[\"manual-reports\"],\
\"evidence_ids\":[\"$evidence_id\"],\"opportunity_score\":30}]}" | jq -e '.run.status == "synthesized"' >/dev/null
workspace="$(call create_opportunity_workspace "{\"run_id\":\"$run\",\"opportunity_id\":\"reporting-saas\"}" | jq \
  -r '.workspace.workspace_id')"
call get_opportunity_execution_pack "{\"workspace_id\":\"$workspace\"}" | jq \
  -e '.workspace.stage == "discovery"' >/dev/null
claim_check="$(call claim_research_job '{}')"
if ! printf '%s' "$claim_check" | jq -e '.job == null' >/dev/null; then
  echo "Unexpected claim after semantic analysis:" >&2
  printf '%s' "$claim_check" | jq -c '{job_id:.job.job_id,status:.job.status}' >&2
  exit 1
fi
# An interrupted MCP host must not strand its claimed job forever.
orphan_job="$(call create_research_job '{"topic":"Abandoned host recovery"}' | jq -r '.job.job_id')"
call claim_research_job '{"harness":"host-one"}' | jq -e --arg \
  id "$orphan_job" '.job.job_id == $id and .job.claimed_by == "host-one"' >/dev/null
call claim_research_job '{"harness":"host-two"}' | jq -e '.job == null' >/dev/null
sql "UPDATE mcp_records SET payload=jsonb_set(payload,'{lease_until}',to_jsonb((now()-interval '1 \
minute')::text)) WHERE owner_id='$owner' AND kind='research-job' AND id='$orphan_job'" >/dev/null
call claim_research_job '{"harness":"host-two"}' | jq -e --arg \
  id "$orphan_job" '.job.job_id == $id and .job.claimed_by == "host-two"' >/dev/null
call start_scrape_session "{\"job_id\":\"$job\",\"claim_token\":\"$claim_token\"}" | jq \
  -e '.session.status == "running"' >/dev/null
call add_scrape_candidates \
  "{\"job_id\":\"$job\",\"claim_token\":\"$claim_token\",\
\"candidates\":[{\"url\":\"https://example.com/review-one\"}]}" | jq \
  -e '.session.candidates | length == 1' >/dev/null
call get_next_scrape_batch "{\"job_id\":\"$job\",\"claim_token\":\"$claim_token\",\"limit\":1}" | jq \
  -e '.batch | length == 1' >/dev/null
call record_scrape_page_result \
  "{\"job_id\":\"$job\",\"claim_token\":\"$claim_token\",\"url\":\"https://example.com/review-one\",\
\"evidence_added\":1}" | jq \
  -e '.session.pages | length == 1' >/dev/null
call get_scrape_session_summary "{\"job_id\":\"$job\"}" | jq -e '.pages | length == 1' >/dev/null
call get_research_quality_summary "{\"run_id\":\"$run\"}" | jq -e '.clusters == 1' >/dev/null
call get_market_sizing_pack "{\"run_id\":\"$run\"}" | jq -e '.opportunities | length == 1' >/dev/null
call submit_market_sizing_assessment "{\"run_id\":\"$run\",\"opportunity_id\":\"reporting-saas\",\
\"sources\":[{\"url\":\"https://example.com/pricing\"}]}" | jq \
  -e '.sizing.opportunity_id == "reporting-saas"' >/dev/null
call submit_opportunity_strategy \
  "{\"workspace_id\":\"$workspace\",\"icp\":\"Small operations teams\",\"wedge\":\"Faster recurring reports\"}" | jq \
  -e '.workspace.strategy.icp == "Small operations teams"' >/dev/null
experiment="$(call create_validation_experiment \
  "{\"workspace_id\":\"$workspace\",\"type\":\"customer-interviews\",\"hypothesis\":\"Teams need it\"}" | jq \
  -r '.experiment_id')"
call record_validation_result \
  "{\"workspace_id\":\"$workspace\",\"experiment_id\":\"$experiment\",\"verdict\":\"supports\",\
\"status\":\"complete\"}" | jq \
  -e '.workspace.experiments[0].verdict == "supports"' >/dev/null
call submit_build_spec \
  "{\"workspace_id\":\"$workspace\",\"product_name\":\"Workflow Report\",\"one_liner\":\"Eliminate reporting \
drudgery\"}" | jq \
  -e '.workspace.build_spec.product_name == "Workflow Report"' >/dev/null
call submit_gtm_plan "{\"workspace_id\":\"$workspace\",\"offer\":\"Design partner pilot\"}" | jq \
  -e '.workspace.gtm_plan.offer == "Design partner pilot"' >/dev/null
call set_opportunity_stage "{\"workspace_id\":\"$workspace\",\"stage\":\"building\"}" | jq \
  -e '.workspace.stage == "building"' >/dev/null
printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | docker \
  compose exec -T -e "PAIN_MCP_API_KEY=$token" -e PAIN_PLATFORM_API_URL=http://web:3000 \
  api /usr/local/bin/pain-intelligence-api --mcp | jq -e '.result.tools | length == 58' >/dev/null
# Browser session mutations require an exact same-origin Origin header.
session='local_integration_cookie_do_not_use'
session_hash="$(printf '%s' "$session" | sha256sum | cut -c1-64)"
sql "INSERT INTO user_sessions(token_hash,user_id,expires_at) VALUES('$session_hash','$owner',now()+interval \
'1 day')" >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/projects" \
  -H "Cookie: pil_session=$session" -H 'Origin: https://evil.example' \
  -H 'Content-Type: application/json' --data '{"name":"blocked project"}')" = 403
test "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/projects" \
  -H "Cookie: pil_session=$session" \
  -H 'Content-Type: application/json' --data '{"name":"missing origin"}')" = 403
# Source adapter discovery requires no credentials; collection always requires a session.
curl --fail-with-body -sS "$api/sources" | jq -e '.sources | length == 8' >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/sources/collect" -H 'Content-Type: application/json' \
  --data '{"source":"github","query":"feature request"}')" = 401
# Local-only authentication works without Reddit OAuth credentials.
local_result="$(curl --fail-with-body -sS -X POST "$api/auth/local" \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  --data '{"token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}' -c /tmp/osdl-local-cookie.txt)"
printf '%s' "$local_result" | jq -e '.ok == true' >/dev/null
curl --fail-with-body -sS "$api/me" -b /tmp/osdl-local-cookie.txt | jq \
  -e '.local_account == true and .reddit_connected == false' >/dev/null
test \
  "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/auth/local" \
    -H 'Origin: https://evil.example' -H 'Content-Type: application/json' \
    --data '{"token":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}')" = 403
test \
  "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/sources/collect" \
    -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
    -H 'Content-Type: application/json' \
    --data '{"source":"medium","query":"../../etc/passwd"}')" = 400
test \
  "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/sources/collect" \
    -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
    -H 'Content-Type: application/json' \
    --data '{"source":"discourse","query":"bugs","target":"localhost"}')" = 400
test \
  "$(curl -s -o /dev/null -w '%{http_code}' -X POST "$api/sources/collect" \
    -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
    -H 'Content-Type: application/json' \
    --data '{"source":"youtube","target":"invalid/video/id"}')" = 400
curl --fail-with-body -sS -X POST "$api/projects" -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  --data '{"name":"Permitted research project"}' | jq -e '.name == "Permitted research project"' >/dev/null
test "$(sql "SELECT count(*) FROM information_schema.tables WHERE table_name='reddit_api_budgets'")" = 1
# Deleting a project reassigns its evidence (rather than deleting unrelated research).
project_id="$(curl --fail-with-body -sS "$api/projects" -H "Cookie: pil_session=$session" | jq -r '.[0].id')"
# Upsert of manually collected evidence must update its project assignment and source URL.
manual_id="$(curl --fail-with-body -sS -X POST "$api/evidence" \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
  -H 'Content-Type: application/json' --data \
  "{\"source_kind\":\"manual\",\"source_id\":\"ci-manual-upsert\",\
\"source_url\":\"https://example.com/old-source\",\"title\":\"Initial link\",\
\"body\":\"Accessibility issue\",\"project_id\":\"$project_id\"}" | jq -r '.id')"
curl --fail-with-body -sS -X POST "$api/evidence" -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  --data \
  '{
  "source_kind": "manual",
  "source_id": "ci-manual-upsert",
  "source_url": "https://example.com/current-source",
  "title": "Updated link",
  "body": "Accessibility issue",
  "project_id": null
}' | jq -e --arg id "$manual_id" '.id == $id' >/dev/null
test "$(sql "SELECT (project_id='$project_id')::text||':'||source_url FROM evidence WHERE id='$manual_id'")" \
  = 'true:https://example.com/current-source'
test \
  "$(sql \
    "SELECT count(*) FROM evidence_projects WHERE owner_id='$owner' AND evidence_id='$manual_id' AND \
project_id='$project_id'")" \
  = 1
second_project="$(curl --fail-with-body -sS -X POST "$api/projects" \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
  -H 'Content-Type: application/json' --data '{"name":"Independent project B"}' | jq \
  -r '.id')"
curl --fail-with-body -sS -X POST "$api/evidence" -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  --data \
  "{\"source_kind\":\"manual\",\"source_id\":\"ci-manual-upsert\",\
\"source_url\":\"https://example.com/current-source\",\"body\":\"manual spreadsheet \
accessibility issue\",\"project_id\":\"$second_project\"}" | jq -e --arg id "$manual_id" '.id == $id' >/dev/null
test "$(sql "SELECT count(*) FROM evidence_projects WHERE owner_id='$owner' AND evidence_id='$manual_id'")" = 2
curl --fail-with-body -sS "$api/evidence?project_id=$project_id" \
  -H "Cookie: pil_session=$session" | jq -e --arg id "$manual_id" \
  'any(.[]; .id == $id)' >/dev/null
curl --fail-with-body -sS "$api/evidence?project_id=$second_project" \
  -H "Cookie: pil_session=$session" | jq -e --arg id "$manual_id" \
  'any(.[]; .id == $id)' >/dev/null
curl --fail-with-body -sS -X POST "$api/research/analyze" -H \
  "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
  -H 'Content-Type: application/json' --data \
  "{\"topic\":\"Recurring community needs that open-source projects could address\",\
\"project_id\":\"$second_project\"}" | jq -e '.report.clusters | length > 0' >/dev/null
curl --fail-with-body -sS "$api/opportunities?project_id=$second_project" \
  -H "Cookie: pil_session=$session" | jq -e 'length > 0' >/dev/null
curl --fail-with-body -sS "$api/opportunities?project_id=$project_id" \
  -H "Cookie: pil_session=$session" | jq -e 'length == 0' >/dev/null
# Opportunity review is a human decision, separate from inferred demand.
candidate="$(curl --fail-with-body -sS "$api/opportunities?project_id=$second_project" \
  -H "Cookie: pil_session=$session" | jq -r '.[0].id')"
curl --fail-with-body -sS "$api/opportunities/$candidate/brief" \
  -H "Cookie: pil_session=$session" | jq -e '.referenced_count > 0 and .cluster != null' >/dev/null
curl --fail-with-body -sS -X PUT "$api/opportunities/$candidate/review" \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
  -H 'Content-Type: application/json' \
  --data '{"decision":"needs_validation","validation_notes":"Validate the need against independently gathered information.","existing_solutions":"Fictional spreadsheet workaround"}' \
  | jq -e '.saved == true' >/dev/null
curl --fail-with-body -sS "$api/opportunities?project_id=$second_project" \
  -H "Cookie: pil_session=$session" | jq -e --arg id "$candidate" \
  'any(.[]; .id == $id and .decision == "needs_validation" and .existing_solutions == "Fictional spreadsheet workaround")' >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/opportunities/$candidate/brief")" = 401
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT "$api/opportunities/$candidate/review" \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  --data '{"decision":"not_pursuing","validation_notes":"","existing_solutions":""}')" = 401
curl --fail-with-body -sS "$api/projects/$second_project" -X DELETE \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' | jq -e '.deleted == true' >/dev/null
curl --fail-with-body -sS "$api/projects/$project_id" -X DELETE \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' | jq -e '.deleted == true' >/dev/null
curl --fail-with-body -sS "$api/privacy/export" -H "Cookie: pil_session=$session" |
  jq -e \
    '.account.reddit_username == "ci-researcher" and (.projects|length) == 0
      and (.host_records|length) >= 2 and .truncated == false' >/dev/null
# Revocable API credentials are shown only once; revoked credentials cannot call tools.
issued="$(curl --fail-with-body -sS -X POST "$api/mcp/keys" \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
  -H 'Content-Type: application/json' --data '{"label":"CI temporary MCP client"}')"
issued_secret="$(printf '%s' "$issued" | jq -r '.api_key')"
issued_id="$(printf '%s' "$issued" | jq -r '.id')"
[[ "$issued_secret" == pil_* ]]
curl --fail-with-body -sS "$api/mcp/keys" -H "Cookie: pil_session=$session" |
  jq -e '.[] | select(.label=="CI temporary MCP client") | .id' >/dev/null
curl --fail-with-body -sS -X DELETE "$api/mcp/keys/$issued_id" -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' | jq -e '.revoked == true' >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/mcp/execute" \
  -H "Authorization: Bearer $issued_secret" -H 'Content-Type: application/json' \
  --data '{"name":"platform_status","arguments":{}}')" = 401
other="$(sql "INSERT INTO users(reddit_id,reddit_username) VALUES('ci-other-id','ci-other') ON CONFLICT \
(reddit_id) DO UPDATE SET reddit_username=excluded.reddit_username RETURNING id")"
other_token='pil_other_user_key'
other_hash="$(printf '%s' "$other_token" | sha256sum | cut -c1-64)"
sql "INSERT INTO mcp_api_keys(owner_id,key_hash,label) VALUES('$other','$other_hash','Second account')" >/dev/null
other_session='ci-second-user-cookie'
other_session_hash="$(printf '%s' "$other_session" | sha256sum | cut -c1-64)"
sql "INSERT INTO user_sessions(token_hash,user_id,expires_at) VALUES('$other_session_hash','$other',now()+interval '1 hour')" >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/opportunities/$candidate/brief" \
  -H "Cookie: pil_session=$other_session")" = 404
test "$(curl -s -o /dev/null -w '%{http_code}' -X PUT "$api/opportunities/$candidate/review" \
  -H "Cookie: pil_session=$other_session" -H 'Origin: http://localhost:3000' \
  -H 'Content-Type: application/json' \
  --data '{"decision":"not_pursuing","validation_notes":"","existing_solutions":""}')" = 404
curl --fail-with-body -sS "$api/mcp/execute" -H "Authorization: Bearer $other_token" \
  -H 'Content-Type: application/json' --data '{"name":"search_evidence","arguments":{"q":"spreadsheets"}}' | jq \
  -e '.returned == 0' >/dev/null
# Corrupted or expired stored credentials must not block local source-data deletion.
sql \
  "INSERT INTO reddit_credentials(user_id,access_token_ciphertext,expires_at) VALUES('$owner',\
'invalid-local-test-token',now()+interval '1 hour')" >/dev/null
curl --fail-with-body -sS "$api/auth/reddit/disconnect" -X POST -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' | jq -e '.disconnected == true and .provider_token_revoked == false' >/dev/null
test "$(sql "SELECT count(*) FROM reddit_credentials WHERE user_id='$owner'")" = 0
curl --fail-with-body -sS "$api/me" -H "Cookie: pil_session=$session" |
  jq -e '.reddit_connected == false' >/dev/null
evidence_id="$(call search_evidence '{"q":"spreadsheets"}' | jq -r '.items[0].id')"
curl --fail-with-body -sS "$api/evidence/$evidence_id" -X DELETE \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' | jq -e '.deleted == true' >/dev/null
call search_evidence '{"q":"spreadsheets"}' | jq -e '.returned == 0' >/dev/null
# Regression: excluding already annotated evidence must happen before LIMIT 500.
sql "INSERT INTO evidence(owner_id,source_kind,source_id,source_url,title,body)
 SELECT '$owner','manual','ci-page-'||n,'https://example.com/page/'||n,'Other subject','routine content'
 FROM generate_series(1,505) AS g(n)" >/dev/null
pagination_run="$(call start_llm_research_run '{"name":"pagination regression"}' | jq -r '.run.id')"
sql "UPDATE mcp_records SET payload=jsonb_set(payload,'{annotations}',
 (SELECT jsonb_object_agg(id::text,jsonb_build_object('evidence_id',id::text)) FROM
 (SELECT id FROM evidence WHERE owner_id='$owner' ORDER BY collected_at DESC,id DESC LIMIT 500) v))
 WHERE owner_id='$owner' AND kind='llm-run' AND id='$pagination_run'" >/dev/null
call get_llm_evidence_batch "{\"run_id\":\"$pagination_run\"}" | jq -e '.returned > 0' >/dev/null
# Regression: a specific research question excludes unrelated account-wide evidence.
curl --fail-with-body -sS -X POST "$api/evidence" -H "Cookie: pil_session=$session" \
  -H 'Origin: http://localhost:3000' -H 'Content-Type: application/json' \
  --data \
  '{
  "source_kind": "manual",
  "source_id": "ci-specific-topic",
  "source_url": "",
  "title": "Unique analytical signal",
  "body": "uniquesubjectkeyword reliability concern"
}' | jq -e '.id|length > 5' >/dev/null
# Push older relevant evidence beyond the previous 1,000-row pre-filter limit.
sql "INSERT INTO evidence(owner_id,source_kind,source_id,source_url,title,body)
 SELECT '$owner','manual','ci-newer-noise-'||n,'','Unrelated note','No matching keywords'
 FROM generate_series(1,1005) AS g(n)" >/dev/null
curl --fail-with-body -sS -X POST "$api/research/analyze" -H \
  "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' \
  -H 'Content-Type: application/json' --data '{"topic":"uniquesubjectkeyword"}' | jq \
  -e '.report.evidence_scanned == 1' >/dev/null
call analyze_pain_points '{"q":"%"}' | jq -e '.report.evidence_scanned == 0' >/dev/null
call analyze_pain_points '{"q":"_"}' | jq -e '.report.evidence_scanned == 0' >/dev/null
# Versioned analysis: bounded coverage, real date filters and ephemeral quotations.
call analyze_pain_points '{"since":"2999-01-01T00:00:00Z"}' | jq -e '.report.evidence_scanned == 0' >/dev/null
call analyze_pain_points '{"limit":1}' | jq -e '.report.coverage.omitted_count > 0 and .report.coverage.selected_count == 1' >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/mcp/execute" -H "Authorization: Bearer $token" \
  -H 'Content-Type: application/json' --data '{"name":"analyze_pain_points","arguments":{"since":"invalid-date"}}')" = 400
quote_id="$(sql "INSERT INTO evidence(owner_id,source_kind,source_id,source_url,title,body) VALUES
 ('$owner','manual','ci-quote-quality','','quotequalityfixture','slow fictional example') RETURNING id")"
quote_scan="$(call analyze_pain_points '{"q":"quotequalityfixture","save_as":"quote lifecycle"}')"
echo "$quote_scan" | jq -e '.report.score_version == 2 and .report.clusters[0].supporting_evidence[0].available == true' >/dev/null
quote_scan_id="$(jq -r '.saved.id' <<<"$quote_scan")"
# Stored reports have references/hashes but never hydrated source quotations.
test "$(sql "SELECT (payload->'report'->'clusters'->0 ? 'supporting_evidence')::int FROM mcp_records
 WHERE owner_id='$owner' AND kind='saved-analysis' AND id='$quote_scan_id'")" = 0
# A content change invalidates old excerpts instead of silently rewriting history.
sql "UPDATE evidence SET body='changed fictional text' WHERE id='$quote_id'" >/dev/null
call list_saved_analyses '{}' | jq -e --arg id "$quote_scan_id" '.scans[] | select(.id==$id) | .report.clusters[0].supporting_evidence[0].available == false' >/dev/null
sql "DELETE FROM evidence WHERE id='$quote_id'" >/dev/null
call list_saved_analyses '{}' | jq -e --arg id "$quote_scan_id" '.scans[] | select(.id==$id) | .report.clusters[0].supporting_evidence[0].available == false' >/dev/null
# Even a forged stored reference cannot hydrate another owner's evidence.
foreign_id="$(sql "INSERT INTO evidence(owner_id,source_kind,source_id,source_url,title,body) VALUES
 ('$other','manual','ci-foreign-quote','','foreign synthetic title','foreign synthetic body') RETURNING id")"
foreign_hash="$(printf 'foreign synthetic title\nforeign synthetic body' | sha256sum | cut -c1-64)"
sql "UPDATE mcp_records SET payload=jsonb_set(payload,'{report,clusters,0,example_refs}',
 jsonb_build_array(jsonb_build_object('evidence_id','$foreign_id','content_hash','$foreign_hash')))
 WHERE owner_id='$owner' AND kind='saved-analysis' AND id='$quote_scan_id'" >/dev/null
call list_saved_analyses '{}' | jq -e --arg id "$quote_scan_id" '.scans[] | select(.id==$id) | .report.clusters[0].supporting_evidence[0].available == false' >/dev/null
curl --fail-with-body -sS "$api/privacy/delete-data" -X POST \
  -H "Cookie: pil_session=$session" -H 'Origin: http://localhost:3000' | jq -e '.deleted == true' >/dev/null
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/me" -H "Cookie: pil_session=$session")" = 401
test "$(curl -s -o /dev/null -w '%{http_code}' "$api/mcp/execute" \
  -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
  --data '{"name":"platform_status","arguments":{}}')" = 401
echo "Rust MCP integration: authorized workflows, 58 schemas, tenant isolation and regression \
checks passed."
