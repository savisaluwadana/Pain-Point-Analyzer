//! Explicitly opt-in local-only high-entropy token authentication.
use crate::{internal, oauth, ApiError, ApiResult, AppState};
use axum::{
    extract::State,
    http::{
        header::{ORIGIN, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{rngs::OsRng, RngCore};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::types::Uuid;

#[derive(Deserialize)]
pub struct LocalCredentials {
    token: String,
}
fn token_matches(input: &str, expected: &str) -> bool {
    let lhs = Sha256::digest(input.as_bytes());
    let rhs = Sha256::digest(expected.as_bytes());
    lhs.iter().zip(rhs.iter()).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
}
pub async fn login(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LocalCredentials>,
) -> ApiResult<Response> {
    let expected = st
        .cfg
        .local_login_token
        .as_deref()
        .ok_or_else(|| ApiError(StatusCode::SERVICE_UNAVAILABLE, "Local login is not enabled".into()))?;
    if headers.get(ORIGIN).and_then(|v| v.to_str().ok()) != Some(st.cfg.web_url.as_str()) {
        return Err(ApiError(StatusCode::FORBIDDEN, "Same-origin login required".into()));
    }
    if body.token.len() != 64 || !token_matches(&body.token, expected) {
        return Err(ApiError(StatusCode::UNAUTHORIZED, "Invalid local access token".into()));
    }
    let mut tx = st.db.begin().await.map_err(internal)?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO users(reddit_id,reddit_username) VALUES('local-only','local-researcher') ON \
            CONFLICT(reddit_id) DO UPDATE SET reddit_username=excluded.reddit_username RETURNING id",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let session = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hex::encode(Sha256::digest(session.as_bytes()));
    sqlx::query("INSERT INTO user_sessions(token_hash,user_id,expires_at) VALUES($1,$2,now()+interval '7 days')")
        .bind(hash)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    let mut response = Json(json!({
        "ok": true,
    }))
    .into_response();
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&oauth::callback_cookie(&st, &session, 604800)).map_err(internal)?,
    );
    Ok(response)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_access_key_matching() {
        assert!(token_matches(&"a".repeat(64), &"a".repeat(64)));
        assert!(!token_matches(&"a".repeat(64), &"b".repeat(64)));
    }
}
