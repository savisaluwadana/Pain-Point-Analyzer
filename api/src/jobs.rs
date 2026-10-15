use crate::{oauth,reddit,research::{assert_project,persist_posts},AppState,ApiResult,bad,internal};
use axum::{extract::State,http::{HeaderMap,StatusCode},Json};
use chrono::{DateTime,Utc};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sqlx::{FromRow,types::Uuid};
use tokio::time::{sleep,Duration};

#[derive(Deserialize)]
pub struct JobInput{subreddit:String,project_id:Option<Uuid>}
#[derive(Serialize,FromRow)]
pub struct JobRow{id:Uuid,subreddit:String,status:String,attempts:i32,last_error:Option<String>,created_at:DateTime<Utc>}
pub async fn list(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Json<Vec<JobRow>>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let rows=sqlx::query_as("SELECT id,subreddit,status,attempts,last_error,created_at FROM research_jobs WHERE owner_id=$1 ORDER BY created_at DESC LIMIT 100").bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 Ok(Json(rows))
}
pub async fn create(State(st):State<AppState>,headers:HeaderMap,Json(body):Json<JobInput>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 if !st.cfg.reddit_approved {return Err(crate::ApiError(axum::http::StatusCode::SERVICE_UNAVAILABLE,"Reddit API access is not approved for this deployment".into()));}
 let connected:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM reddit_credentials WHERE user_id=$1)")
  .bind(uid).fetch_one(&st.db).await.map_err(internal)?;
 if !connected{return Err(crate::bad("Reconnect Reddit before queuing API collection jobs"));}
 assert_project(&st,uid,body.project_id).await?;
 if body.subreddit.len()<2||body.subreddit.len()>21||!body.subreddit.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'){return Err(bad("Invalid subreddit"));}
 let id:Uuid=sqlx::query_scalar("INSERT INTO research_jobs(owner_id,project_id,subreddit) VALUES($1,$2,$3) RETURNING id")
   .bind(uid).bind(body.project_id).bind(&body.subreddit).fetch_one(&st.db).await.map_err(internal)?;
 Ok(Json(json!({"job_id":id,"status":"queued"})))
}
pub async fn worker(st:AppState){
 loop {
  if let Err(err)=work_once(&st).await {tracing::error!(error=%err.1,"research worker iteration failed");}
  sleep(Duration::from_secs(5)).await;
 }
}
// Retry temporary network/provider failures, but not approval, credential, or
// input errors. A refused 403 is not a signal to switch collection endpoints.
fn retryable_reddit_error(error:&crate::ApiError)->bool{
 matches!(error.0,StatusCode::TOO_MANY_REQUESTS|StatusCode::BAD_GATEWAY|
  StatusCode::INTERNAL_SERVER_ERROR|StatusCode::GATEWAY_TIMEOUT)
}
async fn work_once(st:&AppState)->ApiResult<()>{
 let item:Option<(Uuid,Uuid,String,Option<Uuid>,Uuid)>=sqlx::query_as(
  "UPDATE research_jobs SET status='running',lease_token=gen_random_uuid(),lease_expires_at=now()+interval '2 minutes',attempts=attempts+1 WHERE id=(SELECT id FROM research_jobs WHERE (status='queued' AND next_attempt_at<=now()) OR (status='running' AND lease_expires_at<now() AND attempts<3) ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1) RETURNING id,owner_id,subreddit,project_id,lease_token")
   .fetch_optional(&st.db).await.map_err(internal)?;
 let Some((id,uid,sub,project,lease))=item else{return Ok(());};
 let outcome=async {
    let posts=reddit::fetch_posts(st,uid,&sub,25).await?;
    persist_posts(st,uid,posts,project).await
 }.await;
 match outcome {
   Ok(_)=>{sqlx::query("UPDATE research_jobs SET status='complete',lease_token=NULL,lease_expires_at=NULL,finished_at=now() WHERE id=$1 AND lease_token=$2").bind(id).bind(lease).execute(&st.db).await.map_err(internal)?;},
   Err(error)=>{
     let retryable=retryable_reddit_error(&error);
     sqlx::query("UPDATE research_jobs SET status=CASE WHEN attempts>=3 OR NOT $5 THEN 'failed' ELSE 'queued' END,lease_token=NULL,lease_expires_at=NULL,last_error=$3,next_attempt_at=now() + (CASE WHEN $4 THEN interval '15 minutes' ELSE (attempts * interval '1 minute') END),finished_at=CASE WHEN attempts>=3 OR NOT $5 THEN now() ELSE NULL END WHERE id=$1 AND lease_token=$2")
       .bind(id).bind(lease).bind(error.1).bind(error.0==StatusCode::TOO_MANY_REQUESTS).bind(retryable).execute(&st.db).await.map_err(internal)?;
   }
 }
 Ok(())
}

pub async fn cleanup(st:AppState){
 let days: i32=std::env::var("REDDIT_RETENTION_DAYS").ok().and_then(|s|s.parse().ok()).unwrap_or(7).clamp(1,30);
 loop {
   if let Err(err)=sqlx::query("DELETE FROM evidence WHERE source_kind IN ('reddit','reddit-comment','reddit-archive','reddit-archive-comment') AND collected_at < now() - ($1 * interval '1 day')").bind(days).execute(&st.db).await{
     tracing::error!(error=%err,"retention cleanup failed");
   }
   let days:i32=std::env::var("SOURCE_RETENTION_DAYS").ok().and_then(|s|s.parse().ok()).unwrap_or(7).clamp(1,30);
   if let Err(err)=sqlx::query("DELETE FROM evidence WHERE source_kind IN ('github','hackernews','stackoverflow','bluesky','youtube','discourse','medium') AND collected_at < now() - ($1 * interval '1 day')")
     .bind(days).execute(&st.db).await {tracing::error!(error=%err,"source evidence cleanup failed");}
   let _=sqlx::query("DELETE FROM user_sessions WHERE expires_at<now()").execute(&st.db).await;
   // A worker killed during its final lease must not leave the job stuck forever.
   let _=sqlx::query("UPDATE research_jobs SET status='failed',lease_token=NULL,lease_expires_at=NULL,finished_at=now(),last_error=COALESCE(last_error,'Worker lease expired after final retry') WHERE status='running' AND lease_expires_at<now() AND attempts>=3").execute(&st.db).await;
   sleep(Duration::from_secs(3600)).await;
 }
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn permanent_access_denials_do_not_retry(){
  for status in [StatusCode::FAILED_DEPENDENCY,StatusCode::UNAUTHORIZED,
                 StatusCode::FORBIDDEN,StatusCode::SERVICE_UNAVAILABLE,StatusCode::BAD_REQUEST]{
   assert!(!retryable_reddit_error(&crate::ApiError(status,"blocked".into())));
  }
 }
 #[test]
 fn temporary_provider_failures_can_retry(){
  for status in [StatusCode::TOO_MANY_REQUESTS,StatusCode::BAD_GATEWAY,
                 StatusCode::INTERNAL_SERVER_ERROR,StatusCode::GATEWAY_TIMEOUT]{
   assert!(retryable_reddit_error(&crate::ApiError(status,"temporary".into())));
  }
 }
}
