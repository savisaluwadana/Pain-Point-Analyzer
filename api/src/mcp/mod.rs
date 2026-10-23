mod advanced;
mod evidence;
pub mod state;
mod workflows;
use crate::{ApiResult, AppState};
use axum::{extract::State, http::HeaderMap, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

fn catalog() -> Vec<Value> {
    // Embedded repository data, never user/network input; CI validates this catalog.
    serde_json::from_str(include_str!("../mcp_catalog.json")).expect("validated MCP schema catalog")
}
#[derive(Deserialize)]
pub struct ToolRequest {
    pub name: String,
    #[serde(default)]
    pub arguments: Value,
}
pub async fn execute(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ToolRequest>,
) -> ApiResult<Json<Value>> {
    let owner = state::credential_user(&st, &headers).await?;
    let schemas = catalog();
    let spec = schemas.iter().find(|x| x["name"].as_str() == Some(input.name.as_str()));
    let Some(spec) = spec else {
        return Err(crate::bad("Unknown MCP tool"));
    };
    if !input.arguments.is_object() && !input.arguments.is_null() {
        return Err(crate::bad("Tool arguments must be an object"));
    }
    let a = if input.arguments.is_null() {
        json!({})
    } else {
        input.arguments
    };
    let required = spec["inputSchema"]["required"].as_array().cloned().unwrap_or_default();
    for field in &required {
        let Some(key) = field.as_str() else { continue };
        if a.get(key).is_none_or(Value::is_null) {
            return Err(crate::bad(&format!("Missing required field {key}")));
        }
    }
    let result = workflows::dispatch(&st, owner, &input.name, &a).await?;
    Ok(Json(result))
}

pub async fn stdio() -> anyhow::Result<()> {
    let api = env::var("PAIN_PLATFORM_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    let key = env::var("PAIN_MCP_API_KEY")
        .map_err(|_| anyhow::anyhow!("PAIN_MCP_API_KEY must be configured; create a revocable key in the dashboard"))?;
    let url = format!("{}/api/mcp/execute", api.trim_end_matches('/'));
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()?;
    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let mut stdout = io::stdout();
    while let Some(line) = lines.next_line().await? {
        if line.len() > 1024 * 1024 {
            tracing::warn!("MCP line exceeds 1MiB");
            continue;
        }
        let incoming: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error=%e,"Invalid MCP JSON-RPC");
                continue;
            }
        };
        let Some(id) = incoming.get("id").filter(|x| !x.is_null()).cloned() else {
            continue;
        };
        let method = incoming["method"].as_str().unwrap_or_default();
        let params = &incoming["params"];
        let reply = match method {
            "initialize" => json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {
                    "tools": {
                        "listChanged": false,
                    },
                },
                "serverInfo": {
                    "name": "pain-intelligence-rust",
                    "version": "2.0.0",
                },
            }),
            "ping" => json!({}),
            "tools/list" => json!({
                "tools": catalog(),
            }),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));
                let response = http
                    .post(&url)
                    .bearer_auth(&key)
                    .json(&json!({
                        "name": name,
                        "arguments": arguments,
                    }))
                    .send()
                    .await;
                match response {
                    Ok(res) => {
                        let status = res.status();
                        let body: Value = res.json().await.unwrap_or_else(|_| {
                            json!({
                                "error": "Invalid server response",
                            })
                        });
                        if status.is_success() {
                            json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&body).unwrap_or_default(),
                                }],
                                "structuredContent": body,
                                "isError": false,
                            })
                        } else {
                            let msg = body["error"].as_str().unwrap_or("Tool failed");
                            json!({
                                "content": [{
                                    "type": "text",
                                    "text": msg,
                                }],
                                "isError": true,
                            })
                        }
                    }
                    Err(error) => {
                        tracing::error!(%error,"MCP host could not reach Rust API");
                        json!({
                            "content": [{
                                "type": "text",
                                "text": "Rust API unavailable",
                            }],
                            "isError": true,
                        })
                    }
                }
            }
            "resources/list" => json!({ "resources": [] }),
            "prompts/list" => json!({ "prompts": [] }),
            _ => json!({
                "content": [{
                    "type": "text",
                    "text": format!("Unsupported method: {method}"),
                }],
                "isError": true,
            }),
        };
        let envelope = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": reply,
        });
        stdout.write_all(serde_json::to_string(&envelope)?.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}

pub async fn user_runs(State(st): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<Value>>> {
    let owner = crate::oauth::user_id(&st, &headers).await?;
    Ok(Json(state::list(&st, owner, "llm-run", 100).await?))
}
pub async fn user_workspaces(State(st): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<Value>>> {
    let owner = crate::oauth::user_id(&st, &headers).await?;
    Ok(Json(state::list(&st, owner, "workspace", 100).await?))
}
pub async fn user_research_jobs(State(st): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<Value>>> {
    let owner = crate::oauth::user_id(&st, &headers).await?;
    Ok(Json(
        state::list(&st, owner, "research-job", 100)
            .await?
            .iter()
            .map(state::public_job)
            .collect(),
    ))
}
