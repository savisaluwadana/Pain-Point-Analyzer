use crate::{bad, internal, oauth, unauthorized, ApiError, ApiResult, AppState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{types::Uuid, FromRow};

fn digest(raw: &str) -> String {
    hex::encode(Sha256::digest(raw.as_bytes()))
}

pub(crate) async fn credential_user(st: &AppState, headers: &HeaderMap) -> ApiResult<Uuid> {
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(unauthorized)?;
    if !token.starts_with("pil_") || token.len() > 200 {
        return Err(unauthorized());
    }
    let id: Option<Uuid> =
        sqlx::query_scalar("SELECT owner_id FROM mcp_api_keys WHERE key_hash=$1 AND revoked_at IS NULL")
            .bind(digest(token))
            .fetch_optional(&st.db)
            .await
            .map_err(internal)?;
    let id = id.ok_or_else(unauthorized)?;
    let _ = sqlx::query("UPDATE mcp_api_keys SET last_used_at=now() WHERE key_hash=$1")
        .bind(digest(token))
        .execute(&st.db)
        .await;
    Ok(id)
}
#[derive(Deserialize)]
pub struct NewKey {
    label: String,
}
pub async fn create_key(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<NewKey>,
) -> ApiResult<Json<Value>> {
    let owner = oauth::user_id(&st, &headers).await?;
    let label = req.label.trim();
    if label.is_empty() || label.len() > 100 {
        return Err(bad("Key label must be 1-100 characters"));
    }
    let mut bits = [0u8; 32];
    OsRng.fill_bytes(&mut bits);
    let secret = format!("pil_{}", URL_SAFE_NO_PAD.encode(bits));
    let mut tx = st.db.begin().await.map_err(internal)?;
    // Serialize credential issuance for this account to prevent key-sprawl races.
    let _: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE id=$1 FOR UPDATE")
        .bind(owner)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
    let active: i64 = sqlx::query_scalar("SELECT count(*) FROM mcp_api_keys WHERE owner_id=$1 AND revoked_at IS NULL")
        .bind(owner)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
    if active >= 20 {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Revoke an existing API key before creating more than 20 active keys".into(),
        ));
    }
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO mcp_api_keys(owner_id,key_hash,label) VALUES($1,$2,$3) RETURNING id")
            .bind(owner)
            .bind(digest(&secret))
            .bind(label)
            .fetch_one(&mut *tx)
            .await
            .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(Json(json!({
        "id": id,
        "label": label,
        "api_key": secret,
        "notice": "Copy this key now; it will not be shown again.",
    })))
}
#[derive(Serialize, FromRow)]
pub struct KeyRow {
    id: Uuid,
    label: String,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}
pub async fn list_keys(State(st): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Vec<KeyRow>>> {
    let owner = oauth::user_id(&st, &headers).await?;
    let rows = sqlx::query_as(
        "SELECT id,label,created_at,last_used_at FROM mcp_api_keys WHERE owner_id=$1 AND revoked_at IS NULL ORDER \
            BY created_at DESC LIMIT 50",
    )
    .bind(owner)
    .fetch_all(&st.db)
    .await
    .map_err(internal)?;
    Ok(Json(rows))
}
pub async fn revoke_key(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let owner = oauth::user_id(&st, &headers).await?;
    let result =
        sqlx::query("UPDATE mcp_api_keys SET revoked_at=now() WHERE id=$1 AND owner_id=$2 AND revoked_at IS NULL")
            .bind(id)
            .bind(owner)
            .execute(&st.db)
            .await
            .map_err(internal)?;
    if result.rows_affected() == 0 {
        return Err(ApiError(StatusCode::NOT_FOUND, "Key not found".into()));
    }
    Ok(Json(json!({
        "revoked": true,
    })))
}
#[derive(Clone)]
pub(crate) struct Record {
    pub value: Value,
    pub version: i64,
}
pub(crate) fn public_job(value: &Value) -> Value {
    let mut copy = value.clone();
    if let Some(map) = copy.as_object_mut() {
        map.remove("claim_token");
    }
    copy
}
pub(crate) async fn read(st: &AppState, user: Uuid, kind: &str, id: &str) -> ApiResult<Record> {
    let pair: Option<(Value, i64)> =
        sqlx::query_as("SELECT payload,version FROM mcp_records WHERE owner_id=$1 AND kind=$2 AND id=$3")
            .bind(user)
            .bind(kind)
            .bind(id)
            .fetch_optional(&st.db)
            .await
            .map_err(internal)?;
    let (value, version) =
        pair.ok_or_else(|| ApiError(StatusCode::NOT_FOUND, format!("{kind} record was not found")))?;
    Ok(Record { value, version })
}
pub(crate) async fn put(st: &AppState, user: Uuid, kind: &str, id: &str, value: &Value) -> ApiResult<()> {
    let id = id.trim();
    if id.is_empty() || id.len() > 200 || kind.len() > 60 {
        return Err(bad("Invalid record identifier"));
    }
    sqlx::query("INSERT INTO mcp_records(owner_id,kind,id,payload) VALUES($1,$2,$3,$4)")
        .bind(user)
        .bind(kind)
        .bind(id)
        .bind(value)
        .execute(&st.db)
        .await
        .map_err(internal)?;
    Ok(())
}
pub(crate) async fn save(st: &AppState, user: Uuid, kind: &str, id: &str, record: Record) -> ApiResult<()> {
    let count = sqlx::query(
        "UPDATE mcp_records SET payload=$4,version=version+1,updated_at=now() WHERE owner_id=$1 AND kind=$2 AND \
            id=$3 AND version=$5",
    )
    .bind(user)
    .bind(kind)
    .bind(id)
    .bind(record.value)
    .bind(record.version)
    .execute(&st.db)
    .await
    .map_err(internal)?
    .rows_affected();
    if count == 0 {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "Concurrent update; reload the record and retry".into(),
        ));
    }
    Ok(())
}
pub(crate) async fn list(st: &AppState, user: Uuid, kind: &str, limit: i64) -> ApiResult<Vec<Value>> {
    let rows: Vec<(Value,)> = sqlx::query_as(
        "SELECT payload FROM mcp_records WHERE owner_id=$1 AND kind=$2 ORDER BY updated_at DESC LIMIT $3",
    )
    .bind(user)
    .bind(kind)
    .bind(limit.clamp(1, 500))
    .fetch_all(&st.db)
    .await
    .map_err(internal)?;
    Ok(rows.into_iter().map(|(v,)| v).collect())
}
pub(crate) fn string<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
pub(crate) fn require<'a>(v: &'a Value, key: &str) -> ApiResult<&'a str> {
    let value = string(v, key);
    if value.is_empty() || value.len() > 500 {
        return Err(bad(&format!("Missing or invalid {key}")));
    }
    Ok(value)
}
pub(crate) fn limited_array(v: &Value, key: &str, max: usize) -> ApiResult<Vec<Value>> {
    let arr = v.get(key).and_then(Value::as_array).cloned().unwrap_or_default();
    if arr.len() > max {
        return Err(bad(&format!("Too many {key} (limit {max})")));
    }
    Ok(arr)
}
