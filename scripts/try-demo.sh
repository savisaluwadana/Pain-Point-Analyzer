#!/usr/bin/env bash
# Reproducible, explicitly fictional demonstration. No external provider API requests.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

for dependency in curl jq; do
  if ! command -v "$dependency" >/dev/null 2>&1; then
    echo "Missing required command: $dependency" >&2
    exit 1
  fi
done

origin="${OSDL_BASE_URL:-http://localhost:3000}"
case "$origin" in
http://localhost:3000) ;;
*)
  echo "The demo expects the default http://localhost:3000 WEB_URL and origin (CSRF protection \
requires an exact match)" >&2
  exit 1
  ;;
esac

access_token="${LOCAL_LOGIN_TOKEN:-}"
if [[ -z "$access_token" && -f .env ]]; then
  access_token="$(sed -n 's/^LOCAL_LOGIN_TOKEN=//p' .env | tail -n 1 | tr -d '\r')"
fi
if [[ ! "$access_token" =~ ^[[:xdigit:]]{64}$ ]]; then
  echo "Generate LOCAL_LOGIN_TOKEN with 'openssl rand -hex 32', place it in .env and restart Docker \
Compose." >&2
  exit 1
fi

cookie_jar="$(mktemp)"
chmod 600 "$cookie_jar"
trap 'rm -f "$cookie_jar"' EXIT
json_headers=(-H 'Content-Type: application/json' -H "Origin: $origin")

# Authenticate using a local session; never print the token.
printf '%s' "$access_token" | jq -Rs '{token:.}' |
  curl --fail-with-body -sS -o /dev/null -c "$cookie_jar" "${json_headers[@]}" --data-binary @- "$origin/api/auth/local"

project_json="$(curl --fail-with-body -sS -b "$cookie_jar" "${json_headers[@]}" \
  --data '{
  "name": "DEMO (fictional): Community contribution ideas",
  "description": "Synthetic test data only. No statements are quoted from real community members."
}' \
  "$origin/api/projects")"
project_id="$(jq -er '.id' <<<"$project_json")"

count=0
while IFS= read -r observation; do
  payload="$(jq -nc --arg project "$project_id" --argjson item "$observation" \
    \
    '{source_kind:"manual",source_id:("demo:"+$project+":"+$item.id),source_url:"",
      title:$item.title,body:$item.body,project_id:$project}')"
  curl --fail-with-body -sS -o /dev/null -b "$cookie_jar" "${json_headers[@]}" \
    --data "$payload" "$origin/api/evidence"
  count=$((count + 1))
done < <(jq -c '.observations[]' examples/demo-evidence.json)

analysis="$(curl --fail-with-body -sS -b "$cookie_jar" "${json_headers[@]}" \
  --data \
  "$(jq -nc --arg p "$project_id" \
    '{topic:"Synthetic demo: ways open-source projects could help communities",project_id:$p}')" \
  "$origin/api/research/analyze")"
echo "$analysis" | jq -e --argjson n "$count" '.report.evidence_scanned == $n and (.report.clusters|length)>0' >/dev/null

echo "Success: created an explicitly fictional demo project with $count evidence items."
echo "Project ID: $project_id"
echo "Theme groups (heuristic, not verified needs):"
echo "$analysis" | jq -r '.report.clusters[] | "  - \(.category): \(.evidence_count) supporting demo item(s)"'
echo "Open $origin, sign in locally, then choose Evidence Library or Analyze & Discover."
echo "To remove the synthetic items, delete the DEMO project and its evidence in the dashboard."
