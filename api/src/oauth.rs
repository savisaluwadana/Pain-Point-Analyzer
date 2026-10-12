use crate::{AppState,ApiError,ApiResult,bad,internal,unauthorized};
use axum::{extract::{Query,State},http::{header::{COOKIE,SET_COOKIE},HeaderMap,HeaderValue,StatusCode},response::{IntoResponse,Redirect,Response},Json};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD,Engine as _};
use chacha20poly1305::{aead::{Aead,KeyInit,OsRng},ChaCha20Poly1305,Nonce};
use chrono::{DateTime,Utc};
use rand::RngCore;
use serde::{Deserialize,Serialize};
use serde_json::json;
use sha2::{Digest,Sha256};
use sqlx::types::Uuid;
use url::Url;

#[derive(Deserialize)]
pub struct CallbackParams { code:Option<String>, state:Option<String>, error:Option<String> }
#[derive(Deserialize)]
struct TokenResponse { access_token:String, refresh_token:Option<String>, expires_in:i64 }
#[derive(Deserialize)]
struct RedditMe { id:String, name:String }
#[derive(Serialize)]
struct SessionInfo { id:Uuid, reddit_username:String }
fn random_token()->String {let mut bytes=[0u8;32];OsRng.fill_bytes(&mut bytes);URL_SAFE_NO_PAD.encode(bytes)}
fn hash(token:&str)->String {hex::encode(Sha256::digest(token.as_bytes()))}
fn cookie_value(headers:&HeaderMap,name:&str)->Option<String> {
  let cookies=headers.get(COOKIE)?.to_str().ok()?;
  cookies.split(';').find_map(|p|{
    let (k,v)=p.trim().split_once('=')?;
    if k==name {Some(v.to_owned())} else {None}
  })
}
fn set_cookie(name:&str,value:&str,secs:i64,secure:bool)->String{
  format!("{name}={value}; Path=/; Max-Age={secs}; HttpOnly; SameSite=Lax{}",if secure {"; Secure"} else {""})
}
fn callback_cookie(state:&AppState,token:&str,secs:i64)->String {
  set_cookie("pil_session",token,secs,state.cfg.secure_cookies)
}
pub(crate) fn encrypt(state:&AppState,plaintext:&str)->ApiResult<String>{
  let cipher=ChaCha20Poly1305::new_from_slice(&state.cfg.encryption_key).map_err(internal)?;
  let mut nonce=[0u8;12];OsRng.fill_bytes(&mut nonce);
  let ciphertext=cipher.encrypt(Nonce::from_slice(&nonce),plaintext.as_bytes()).map_err(internal)?;
  Ok(format!("{}:{}",URL_SAFE_NO_PAD.encode(nonce),URL_SAFE_NO_PAD.encode(ciphertext)))
}
fn decrypt(state:&AppState,data:&str)->ApiResult<String>{
  let (n,c)=data.split_once(':').ok_or_else(||internal("bad ciphertext"))?;
  let nonce=URL_SAFE_NO_PAD.decode(n).map_err(internal)?;
  if nonce.len()!=12{return Err(internal("bad nonce"));}
  let ct=URL_SAFE_NO_PAD.decode(c).map_err(internal)?;
  let cipher=ChaCha20Poly1305::new_from_slice(&state.cfg.encryption_key).map_err(internal)?;
  let plain=cipher.decrypt(Nonce::from_slice(&nonce),ct.as_slice()).map_err(internal)?;
  String::from_utf8(plain).map_err(internal)
}
pub async fn start(State(st):State<AppState>)->ApiResult<Response>{
  if !st.cfg.reddit_approved {return Err(ApiError(StatusCode::SERVICE_UNAVAILABLE,"Reddit API access must be approved before connecting".into()));}
  if st.cfg.client_id.is_empty() || st.cfg.client_secret.is_empty() || st.cfg.user_agent.is_empty() {
    return Err(ApiError(StatusCode::SERVICE_UNAVAILABLE,"Reddit OAuth application is not configured".into()));
  }
  let state=random_token();
  let mut u=Url::parse("https://www.reddit.com/api/v1/authorize").map_err(internal)?;
  u.query_pairs_mut().append_pair("client_id",&st.cfg.client_id).append_pair("response_type","code")
    .append_pair("state",&state).append_pair("redirect_uri",&st.cfg.redirect_uri)
    .append_pair("duration","permanent").append_pair("scope","identity read");
  let mut resp=Redirect::temporary(u.as_str()).into_response();
  resp.headers_mut().append(SET_COOKIE,HeaderValue::from_str(&set_cookie("pil_oauth_state",&state,600,st.cfg.secure_cookies)).map_err(internal)?);
  Ok(resp)
}
pub async fn callback(State(st):State<AppState>,headers:HeaderMap,Query(q):Query<CallbackParams>)->ApiResult<Response>{
  if q.error.is_some(){return Err(bad("Reddit consent was declined or failed"));}
  let requested=q.state.as_deref().ok_or_else(||bad("Missing OAuth state"))?;
  let cookie=cookie_value(&headers,"pil_oauth_state").ok_or_else(||bad("OAuth state cookie missing or expired"))?;
  if requested.len()<32 || requested!=cookie{return Err(bad("Invalid OAuth state"));}
  let code=q.code.filter(|c|!c.is_empty()).ok_or_else(||bad("Missing authorization code"))?;
  let resp=st.http.post("https://www.reddit.com/api/v1/access_token")
    .basic_auth(&st.cfg.client_id,Some(&st.cfg.client_secret))
    .header("User-Agent",&st.cfg.user_agent)
    .form(&[("grant_type","authorization_code"),("code",&code),("redirect_uri",&st.cfg.redirect_uri)])
    .send().await.map_err(internal)?;
  if !resp.status().is_success(){return Err(ApiError(StatusCode::BAD_GATEWAY,"Reddit rejected OAuth exchange".into()));}
  let token:TokenResponse=resp.json().await.map_err(internal)?;
  let me=st.http.get("https://oauth.reddit.com/api/v1/me")
    .header("Authorization",format!("Bearer {}",token.access_token))
    .header("User-Agent",&st.cfg.user_agent).send().await.map_err(internal)?;
  if !me.status().is_success(){return Err(ApiError(StatusCode::BAD_GATEWAY,"Reddit identity request failed".into()));}
  let profile:RedditMe=me.json().await.map_err(internal)?;
  let mut tx=st.db.begin().await.map_err(internal)?;
  let user_id:Uuid=sqlx::query_scalar("INSERT INTO users(reddit_id,reddit_username) VALUES($1,$2) ON CONFLICT(reddit_id) DO UPDATE SET reddit_username=excluded.reddit_username RETURNING id")
    .bind(&profile.id).bind(&profile.name).fetch_one(&mut *tx).await.map_err(internal)?;
  let access=encrypt(&st,&token.access_token)?;
  let refresh=token.refresh_token.as_ref().map(|x|encrypt(&st,x)).transpose()?;
  sqlx::query("INSERT INTO reddit_credentials(user_id,access_token_ciphertext,refresh_token_ciphertext,expires_at) VALUES($1,$2,$3,now()+($4 * interval '1 second')) ON CONFLICT(user_id) DO UPDATE SET access_token_ciphertext=excluded.access_token_ciphertext,refresh_token_ciphertext=coalesce(excluded.refresh_token_ciphertext,reddit_credentials.refresh_token_ciphertext),expires_at=excluded.expires_at,updated_at=now()")
    .bind(user_id).bind(&access).bind(&refresh).bind(token.expires_in.max(1)).execute(&mut *tx).await.map_err(internal)?;
  let session=random_token();
  sqlx::query("INSERT INTO user_sessions(token_hash,user_id,expires_at) VALUES($1,$2,now()+interval '7 days')")
    .bind(hash(&session)).bind(user_id).execute(&mut *tx).await.map_err(internal)?;
  tx.commit().await.map_err(internal)?;
  let mut resp=Redirect::to(&st.cfg.web_url).into_response();
  resp.headers_mut().append(SET_COOKIE,HeaderValue::from_str(&callback_cookie(&st,&session,7*86400)).map_err(internal)?);
  resp.headers_mut().append(SET_COOKIE,HeaderValue::from_str(&set_cookie("pil_oauth_state","",0,st.cfg.secure_cookies)).map_err(internal)?);
  Ok(resp)
}
pub(crate) async fn user_id(st:&AppState,headers:&HeaderMap)->ApiResult<Uuid>{
  let session=cookie_value(headers,"pil_session").ok_or_else(unauthorized)?;
  sqlx::query_scalar("SELECT user_id FROM user_sessions WHERE token_hash=$1 AND expires_at>now()")
    .bind(hash(&session)).fetch_optional(&st.db).await.map_err(internal)?.ok_or_else(unauthorized)
}
pub async fn me(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Json<serde_json::Value>>{
  let id=user_id(&st,&headers).await?;
  let name:String=sqlx::query_scalar("SELECT reddit_username FROM users WHERE id=$1").bind(id).fetch_one(&st.db).await.map_err(internal)?;
  let reddit_connected:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM reddit_credentials WHERE user_id=$1)")
    .bind(id).fetch_one(&st.db).await.map_err(internal)?;
  Ok(Json(json!({"id":id,"reddit_username":name,"reddit_connected":reddit_connected})))
}
pub async fn logout(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Response>{
  if let Some(session)=cookie_value(&headers,"pil_session"){
    sqlx::query("DELETE FROM user_sessions WHERE token_hash=$1").bind(hash(&session)).execute(&st.db).await.map_err(internal)?;
  }
  let mut resp=Json(json!({"ok":true})).into_response();
  resp.headers_mut().append(SET_COOKIE,HeaderValue::from_str(&callback_cookie(&st,"",0)).map_err(internal)?);
  Ok(resp)
}

pub(crate) async fn revoke_reddit(st:&AppState,id:Uuid)->ApiResult<bool>{
  let row:Option<(String,Option<String>)>=sqlx::query_as(
   "SELECT access_token_ciphertext,refresh_token_ciphertext FROM reddit_credentials WHERE user_id=$1"
  ).bind(id).fetch_optional(&st.db).await.map_err(internal)?;
  let Some((access,refresh))=row else {return Ok(false);};
  let (ciphertext,hint)=match refresh {Some(r)=>(r,"refresh_token"),None=>(access,"access_token")};
  let raw=decrypt(st,&ciphertext)?;
  let response=st.http.post("https://www.reddit.com/api/v1/revoke_token")
   .basic_auth(&st.cfg.client_id,Some(&st.cfg.client_secret))
   .header("User-Agent",&st.cfg.user_agent)
   .form(&[("token",raw.as_str()),("token_type_hint",hint)])
   .send().await.map_err(|err|{
      tracing::warn!(error=%err,"Reddit revocation request failed");
      ApiError(StatusCode::BAD_GATEWAY,"Reddit token revocation could not be confirmed; retry or revoke app access in Reddit settings".into())
   })?;
  if !response.status().is_success(){
    return Err(ApiError(StatusCode::BAD_GATEWAY,"Reddit token revocation could not be confirmed; retry or revoke app access in Reddit settings".into()));
  }
  Ok(true)
}
pub async fn disconnect(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Json<serde_json::Value>>{
  let id=user_id(&st,&headers).await?;
  let remote_revoked=match revoke_reddit(&st,id).await {
    Ok(revoked)=>revoked,
    Err(err)=>{tracing::warn!(error=%err.1,"Reddit revocation not confirmed; completing local disconnect");false}
  };
  let mut tx=st.db.begin().await.map_err(internal)?;
  // Revoke permission to execute any pending or currently leased Reddit API collections.
  sqlx::query("DELETE FROM research_jobs WHERE owner_id=$1")
   .bind(id).execute(&mut *tx).await.map_err(internal)?;
  sqlx::query("DELETE FROM reddit_credentials WHERE user_id=$1")
   .bind(id).execute(&mut *tx).await.map_err(internal)?;
  sqlx::query("DELETE FROM evidence WHERE owner_id=$1 AND source_kind IN ('reddit','reddit-comment')")
   .bind(id).execute(&mut *tx).await.map_err(internal)?;
  tx.commit().await.map_err(internal)?;
  Ok(Json(json!({
   "disconnected":true,
   "provider_token_revoked":remote_revoked,
   "reddit_evidence_deleted":true,
   "notice":"Local Reddit access and source data are removed. If provider revocation is unconfirmed, revoke this app in Reddit settings. Research snapshots/annotations may still reference deleted evidence IDs; delete the account to erase them."
  })))
}

pub(crate) async fn access_token(st:&AppState,id:Uuid)->ApiResult<String>{
  if !st.cfg.reddit_approved {return Err(ApiError(StatusCode::SERVICE_UNAVAILABLE,"Reddit Data API access not approved".into()));}
  let row:Option<(String,Option<String>,DateTime<Utc>)>=sqlx::query_as("SELECT access_token_ciphertext,refresh_token_ciphertext,expires_at FROM reddit_credentials WHERE user_id=$1")
    .bind(id).fetch_optional(&st.db).await.map_err(internal)?;
  let (access,refresh,expires)=row.ok_or_else(unauthorized)?;
  if expires>Utc::now()+chrono::Duration::seconds(60){return decrypt(st,&access);}
  let refresh=refresh.ok_or_else(unauthorized)?;
  let refresh=decrypt(st,&refresh)?;
  let response=st.http.post("https://www.reddit.com/api/v1/access_token")
    .basic_auth(&st.cfg.client_id,Some(&st.cfg.client_secret))
    .header("User-Agent",&st.cfg.user_agent)
    .form(&[("grant_type","refresh_token"),("refresh_token",&refresh)])
    .send().await.map_err(internal)?;
  if !response.status().is_success(){return Err(ApiError(StatusCode::UNAUTHORIZED,"Reddit session expired; reconnect your account".into()));}
  let new_token:TokenResponse=response.json().await.map_err(internal)?;
  let encrypted=encrypt(st,&new_token.access_token)?;
  let replacement=new_token.refresh_token.as_ref().map(|s|encrypt(st,s)).transpose()?;
  sqlx::query("UPDATE reddit_credentials SET access_token_ciphertext=$2,refresh_token_ciphertext=coalesce($3,refresh_token_ciphertext),expires_at=now()+($4 * interval '1 second'),updated_at=now() WHERE user_id=$1")
    .bind(id).bind(encrypted).bind(replacement).bind(new_token.expires_in.max(1)).execute(&st.db).await.map_err(internal)?;
  Ok(new_token.access_token)
}

#[cfg(test)]
mod tests {
 use super::*;
 #[tokio::test]
 async fn token_encryption_round_trip() {
   let db=sqlx::postgres::PgPoolOptions::new().connect_lazy("postgres://test:test@localhost/test").unwrap();
   let cfg=crate::Config{
     client_id:String::new(),client_secret:String::new(),
     redirect_uri:"http://localhost:3000/api/auth/reddit/callback".into(),
     web_url:"http://localhost:3000".into(),user_agent:"test".into(),
     encryption_key:[7u8;32],reddit_approved:false,secure_cookies:false
   };
   let st=crate::AppState{db,http:reqwest::Client::new(),cfg:std::sync::Arc::new(cfg)};
   let raw="reddit-access-token";
   let enc=encrypt(&st,raw).unwrap();
   assert_ne!(enc,raw);
   assert_eq!(decrypt(&st,&enc).unwrap(),raw);
   assert!(decrypt(&st,"invalid").is_err());
 }
 #[test]
 fn session_cookie_is_opaque_and_http_only() {
   let token=random_token();
   let cookie=set_cookie("pil_session",&token,3600,true);
   assert!(cookie.contains("HttpOnly"));
   assert!(cookie.contains("SameSite=Lax"));
   assert!(cookie.contains("Secure"));
   assert!(!token.contains('='));
 }
}
