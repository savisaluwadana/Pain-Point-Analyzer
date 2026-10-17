use crate::{oauth,reddit,scoring::{self,EvidenceText},AppState,ApiResult,bad,internal};
use axum::{extract::{Path,Query,State},http::{HeaderMap,header::{SET_COOKIE,CACHE_CONTROL,CONTENT_DISPOSITION},HeaderValue},response::{IntoResponse,Response},Json};
use chrono::{DateTime,Utc};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use sqlx::{FromRow,types::Uuid};

#[derive(Deserialize)]
pub struct ProjectInput {name:String,description:Option<String>}
#[derive(Serialize,FromRow)]
pub struct Project {id:Uuid,name:String,description:String,created_at:DateTime<Utc>}
pub async fn projects(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Json<Vec<Project>>> {
 let uid=oauth::user_id(&st,&headers).await?;
 let data=sqlx::query_as("SELECT id,name,description,created_at FROM projects WHERE owner_id=$1 ORDER BY created_at DESC LIMIT 100").bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 Ok(Json(data))
}
pub async fn create_project(State(st):State<AppState>,headers:HeaderMap,Json(body):Json<ProjectInput>)->ApiResult<Json<Project>>{
 let uid=oauth::user_id(&st,&headers).await?;
 if body.name.trim().is_empty()||body.name.len()>160{return Err(bad("Project name must be 1-160 characters"));}
 let data=sqlx::query_as("INSERT INTO projects(owner_id,name,description) VALUES($1,$2,$3) RETURNING id,name,description,created_at")
    .bind(uid).bind(body.name.trim()).bind(body.description.unwrap_or_default().chars().take(2000).collect::<String>()).fetch_one(&st.db).await.map_err(internal)?;
 Ok(Json(data))
}
pub async fn delete_project(State(st):State<AppState>,headers:HeaderMap,Path(id):Path<Uuid>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let count=sqlx::query("DELETE FROM projects WHERE owner_id=$1 AND id=$2")
  .bind(uid).bind(id).execute(&st.db).await.map_err(internal)?.rows_affected();
 if count==0{return Err(crate::ApiError(axum::http::StatusCode::NOT_FOUND,"Project not found".into()));}
 Ok(Json(json!({"deleted":true,"project_id":id,"notice":"Project deleted. Associated evidence and reports were preserved as unassigned."})))
}

#[derive(Deserialize)]
pub struct EvidenceInput {source_kind:String,source_id:String,source_url:String,title:Option<String>,body:Option<String>,project_id:Option<Uuid>}
#[derive(Serialize,FromRow)]
pub struct EvidenceRow {id:Uuid,source_kind:String,source_id:String,source_url:String,title:String,body:String,metadata:Value,collected_at:DateTime<Utc>}
#[derive(Deserialize)]
pub struct EvidenceQuery {limit:Option<i64>,project_id:Option<Uuid>}
pub async fn evidence(State(st):State<AppState>,headers:HeaderMap,Query(q):Query<EvidenceQuery>)->ApiResult<Json<Vec<EvidenceRow>>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let rows=sqlx::query_as("SELECT id,source_kind,source_id,source_url,title,body,metadata,collected_at FROM evidence WHERE owner_id=$1 AND ($2::uuid IS NULL OR EXISTS(SELECT 1 FROM evidence_projects ep WHERE ep.owner_id=evidence.owner_id AND ep.evidence_id=evidence.id AND ep.project_id=$2)) ORDER BY collected_at DESC LIMIT $3")
   .bind(uid).bind(q.project_id).bind(q.limit.unwrap_or(100).clamp(1,500)).fetch_all(&st.db).await.map_err(internal)?;
 Ok(Json(rows))
}
pub(crate) async fn assert_project(st:&AppState,uid:Uuid,project:Option<Uuid>)->ApiResult<()>{
 if let Some(id)=project{
  let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE id=$1 AND owner_id=$2)").bind(id).bind(uid).fetch_one(&st.db).await.map_err(internal)?;
  if !exists{return Err(bad("Project not found"));}
 }
 Ok(())
}
pub(crate) async fn link_project(tx:&mut sqlx::Transaction<'_,sqlx::Postgres>,owner:Uuid,evidence:Uuid,project:Option<Uuid>)->ApiResult<()>{
 if let Some(project)=project {
  sqlx::query("INSERT INTO evidence_projects(owner_id,evidence_id,project_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
   .bind(owner).bind(evidence).bind(project).execute(&mut **tx).await.map_err(internal)?;
 }
 Ok(())
}
pub async fn ingest_evidence(State(st):State<AppState>,headers:HeaderMap,Json(body):Json<EvidenceInput>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 assert_project(&st,uid,body.project_id).await?;
 if body.source_kind!="manual"&&body.source_kind!="import"{return Err(bad("Use the authorized Reddit connector for Reddit evidence"));}
 if body.source_id.len()>255||body.source_id.is_empty()||body.source_url.len()>2048{return Err(bad("Invalid evidence identifier or URL"));}
 if !body.source_url.is_empty(){
   let u=url::Url::parse(&body.source_url).map_err(|_|bad("Source URL must use http or https"))?;
   if !matches!(u.scheme(),"https"|"http"){return Err(bad("Source URL must use http or https"));}
   if reddit::is_reddit_host(u.host_str().unwrap_or("")){return Err(bad("Import Reddit evidence through the approved OAuth connector"));}
 }
 let title=body.title.unwrap_or_default().chars().take(500).collect::<String>();
 let text=body.body.unwrap_or_default().chars().take(10000).collect::<String>();
 let mut tx=st.db.begin().await.map_err(internal)?;
 let id:Uuid=sqlx::query_scalar("INSERT INTO evidence(owner_id,project_id,source_kind,source_id,source_url,title,body) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET project_id=COALESCE(evidence.project_id,excluded.project_id),source_url=excluded.source_url,title=excluded.title,body=excluded.body,collected_at=now() RETURNING id")
  .bind(uid).bind(body.project_id).bind(&body.source_kind).bind(&body.source_id).bind(&body.source_url).bind(title).bind(text).fetch_one(&mut *tx).await.map_err(internal)?;
 link_project(&mut tx,uid,id,body.project_id).await?;
 tx.commit().await.map_err(internal)?;
 Ok(Json(json!({"id":id})))
}
#[derive(Deserialize)]
pub struct ScanInput {subreddit:String,project_id:Option<Uuid>,limit:Option<i32>}
pub(crate) async fn persist_posts(st:&AppState,uid:Uuid,posts:Vec<reddit::RedditPost>,project:Option<Uuid>)->ApiResult<usize>{
 let mut tx=st.db.begin().await.map_err(internal)?;
 // Serialize with disconnect so an in-flight API collection cannot recreate deleted evidence.
 let authorized:Option<bool>=sqlx::query_scalar("SELECT true FROM reddit_credentials WHERE user_id=$1 FOR SHARE")
  .bind(uid).fetch_optional(&mut *tx).await.map_err(internal)?;
 if authorized.is_none(){return Err(crate::unauthorized());}
 let count=posts.len();
 for post in posts{
   let url=format!("https://www.reddit.com{}",post.permalink);
   let published=DateTime::from_timestamp(post.created_utc as i64,0);
   let id:Uuid=sqlx::query_scalar("INSERT INTO evidence(owner_id,project_id,source_kind,source_id,source_url,title,body,author,published_at,metadata) VALUES($1,$2,'reddit',$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET project_id=COALESCE(evidence.project_id,excluded.project_id),source_url=excluded.source_url,title=excluded.title,body=excluded.body,author=excluded.author,published_at=excluded.published_at,metadata=excluded.metadata,collected_at=now() RETURNING id")
    .bind(uid).bind(project).bind(format!("t3_{}",post.id)).bind(url)
    .bind(post.title.chars().take(500).collect::<String>()).bind(post.selftext.chars().take(10000).collect::<String>())
    .bind(None::<String>).bind(published).bind(json!({"subreddit":post.subreddit,"score":post.score,"num_comments":post.num_comments}))
    .fetch_one(&mut *tx).await.map_err(internal)?;
   link_project(&mut tx,uid,id,project).await?;
 }
 tx.commit().await.map_err(internal)?;
 Ok(count)
}
pub async fn scan(State(st):State<AppState>,headers:HeaderMap,Json(body):Json<ScanInput>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 assert_project(&st,uid,body.project_id).await?;
 let posts=reddit::fetch_posts(&st,uid,&body.subreddit,body.limit.unwrap_or(25).clamp(1,100)).await?;
 let saved=persist_posts(&st,uid,posts,body.project_id).await?;
 Ok(Json(json!({"saved":saved,"subreddit":body.subreddit})))
}
#[derive(Deserialize)]
pub struct AnalyzeInput {topic:String,project_id:Option<Uuid>}
pub async fn analyze(State(st):State<AppState>,headers:HeaderMap,Json(body):Json<AnalyzeInput>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 assert_project(&st,uid,body.project_id).await?;
 if body.topic.trim().is_empty()||body.topic.len()>500{return Err(bad("Provide a research topic of 1-500 characters"));}
 let terms=scoring::topic_terms(&body.topic);
 let rows:Vec<(Uuid,String,String,String,String)>=sqlx::query_as(
  "SELECT id,source_kind,source_id,title,body FROM evidence WHERE owner_id=$1 AND
   ($2::uuid IS NULL OR EXISTS(SELECT 1 FROM evidence_projects ep WHERE ep.owner_id=evidence.owner_id AND ep.evidence_id=evidence.id AND ep.project_id=$2))
   AND (cardinality($3::text[])=0 OR EXISTS(SELECT 1 FROM unnest($3::text[]) AS keyword(word) WHERE strpos(lower(title || E'\\n' || body),word)>0))
   ORDER BY collected_at DESC LIMIT 1000")
   .bind(uid).bind(body.project_id).bind(&terms).fetch_all(&st.db).await.map_err(internal)?;
 let items=rows.into_iter().map(|(id,source_kind,source_id,title,body)|EvidenceText{id,source_kind,source_id,text:format!("{title}\n{body}")}).collect::<Vec<_>>();
 let relevant=scoring::relevant_to_topic(&items,&body.topic);
  let report=scoring::analyze(&relevant);
 let mut tx=st.db.begin().await.map_err(internal)?;
 let run_id:Uuid=sqlx::query_scalar("INSERT INTO research_runs(owner_id,project_id,topic,report) VALUES($1,$2,$3,$4) RETURNING id")
   .bind(uid).bind(body.project_id).bind(body.topic.trim()).bind(json!(&report)).fetch_one(&mut *tx).await.map_err(internal)?;
 for cluster in &report.clusters {
    sqlx::query("INSERT INTO opportunities(owner_id,run_id,title,problem,score,evidence_refs) VALUES($1,$2,$3,$4,$5,$6)")
     .bind(uid).bind(run_id).bind(&cluster.category)
     .bind(format!("{} independent observed stories; requires validation",cluster.independent_stories))
     .bind(cluster.severity_score).bind(json!(&cluster.evidence_ids)).execute(&mut *tx).await.map_err(internal)?;
 }
 tx.commit().await.map_err(internal)?;
 Ok(Json(json!({"run_id":run_id,"report":report})))
}
#[derive(Serialize,FromRow)]
pub struct RunRow {id:Uuid,project_id:Option<Uuid>,topic:String,report:Value,created_at:DateTime<Utc>}
pub async fn runs(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Json<Vec<RunRow>>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let rows=sqlx::query_as("SELECT id,project_id,topic,report,created_at FROM research_runs WHERE owner_id=$1 ORDER BY created_at DESC LIMIT 100").bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 Ok(Json(rows))
}
#[derive(Serialize,FromRow)]
pub struct OpportunityRow {id:Uuid,project_id:Option<Uuid>,title:String,problem:String,score:f64,evidence_refs:Value,created_at:DateTime<Utc>}
pub async fn opportunities(State(st):State<AppState>,headers:HeaderMap,Query(q):Query<EvidenceQuery>)->ApiResult<Json<Vec<OpportunityRow>>>{
 let uid=oauth::user_id(&st,&headers).await?;
 assert_project(&st,uid,q.project_id).await?;
 let rows=sqlx::query_as("SELECT o.id,r.project_id,o.title,o.problem,o.score,o.evidence_refs,o.created_at FROM opportunities o LEFT JOIN research_runs r ON o.run_id=r.id WHERE o.owner_id=$1 AND ($2::uuid IS NULL OR r.project_id=$2) ORDER BY o.score DESC,o.created_at DESC LIMIT 100")
   .bind(uid).bind(q.project_id).fetch_all(&st.db).await.map_err(internal)?;
 Ok(Json(rows))
}

pub async fn delete_evidence(State(st):State<AppState>,headers:HeaderMap,Path(id):Path<Uuid>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let count=sqlx::query("DELETE FROM evidence WHERE id=$1 AND owner_id=$2")
  .bind(id).bind(uid).execute(&st.db).await.map_err(internal)?.rows_affected();
 if count==0{return Err(crate::ApiError(axum::http::StatusCode::NOT_FOUND,"Evidence not found".into()));}
 Ok(Json(json!({"deleted":true,"id":id,"notice":"Previously saved research runs are snapshots and will not be recalculated automatically."})))
}
pub async fn export_data(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Response>{
 let uid=oauth::user_id(&st,&headers).await?;
 let name:String=sqlx::query_scalar("SELECT reddit_username FROM users WHERE id=$1")
  .bind(uid).fetch_one(&st.db).await.map_err(internal)?;
 let projects:Vec<Value>=sqlx::query_scalar(
   "SELECT to_jsonb(t) FROM (SELECT id,name,description,created_at FROM projects WHERE owner_id=$1 ORDER BY created_at DESC LIMIT 5001) t")
  .bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 let evidence:Vec<Value>=sqlx::query_scalar(
   "SELECT to_jsonb(t) FROM (SELECT id,project_id,source_kind,source_id,source_url,CASE WHEN source_kind IN ('reddit','reddit-comment','reddit-archive','reddit-archive-comment') THEN NULL ELSE title END AS title,CASE WHEN source_kind IN ('reddit','reddit-comment','reddit-archive','reddit-archive-comment') THEN NULL ELSE body END AS body,published_at,collected_at,metadata FROM evidence WHERE owner_id=$1 ORDER BY collected_at DESC LIMIT 5001) t")
  .bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 let runs:Vec<Value>=sqlx::query_scalar(
   "SELECT to_jsonb(t) FROM (SELECT id,project_id,topic,report,status,created_at FROM research_runs WHERE owner_id=$1 ORDER BY created_at DESC LIMIT 5001) t")
  .bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 let opportunities:Vec<Value>=sqlx::query_scalar(
   "SELECT to_jsonb(t) FROM (SELECT id,run_id,title,problem,score,evidence_refs,created_at FROM opportunities WHERE owner_id=$1 ORDER BY created_at DESC LIMIT 5001) t")
  .bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 let records:Vec<Value>=sqlx::query_scalar(
   "SELECT to_jsonb(t) FROM (SELECT kind,id,payload,created_at,updated_at FROM mcp_records WHERE owner_id=$1 ORDER BY updated_at DESC LIMIT 5001) t")
  .bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 let truncated=projects.len()>5000||evidence.len()>5000||runs.len()>5000||opportunities.len()>5000||records.len()>5000;
 let take=|mut items:Vec<Value>|{items.truncate(5000);items};
 let mut response=Json(json!({
  "format":"pain-intelligence-lab-v1",
  "exported_at":Utc::now().to_rfc3339(),
  "account":{"id":uid,"reddit_username":name},
  "projects":take(projects),"evidence":take(evidence),
  "research_runs":take(runs),"opportunities":take(opportunities),"host_records":take(records),
  "truncated":truncated,
  "notes":["OAuth tokens, user sessions, MCP API secrets and Reddit post/comment body text are excluded","Each collection is limited to 5000 records; if truncated=true, contact the operator for a full export"]
 })).into_response();
 response.headers_mut().insert(CACHE_CONTROL,HeaderValue::from_static("private, no-store"));
 response.headers_mut().insert(CONTENT_DISPOSITION,HeaderValue::from_static("attachment; filename=\"pain-intelligence-export.json\""));
 Ok(response)
}

pub async fn delete_data(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Response>{
 let uid=oauth::user_id(&st,&headers).await?;
 let revoked=match oauth::revoke_reddit(&st,uid).await {Ok(v)=>v,Err(err)=>{tracing::warn!(error=%err.1,"Deleting local data even though remote revocation could not be verified");false}};
 sqlx::query("DELETE FROM users WHERE id=$1").bind(uid).execute(&st.db).await.map_err(internal)?;
 let mut response=Json(json!({"deleted":true,"provider_token_revocation_confirmed":revoked,"action":"Your stored account, credentials, evidence, research and sessions were deleted. If provider revocation was not confirmed, revoke the app in Reddit settings."})).into_response();
 response.headers_mut().insert(SET_COOKIE,HeaderValue::from_static("pil_session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"));
 response.headers_mut().insert(CACHE_CONTROL,HeaderValue::from_static("private, no-store"));
 Ok(response)
}

#[derive(Deserialize)]
pub struct ThreadInput {post_id:String,project_id:Option<Uuid>,limit:Option<usize>}
pub async fn thread(State(st):State<AppState>,headers:HeaderMap,Json(body):Json<ThreadInput>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 assert_project(&st,uid,body.project_id).await?;
 let comments=reddit::fetch_comments(&st,uid,&body.post_id,body.limit.unwrap_or(50).clamp(1,100)).await?;
 let count=comments.len();
 let mut tx=st.db.begin().await.map_err(internal)?;
 // Prevent in-flight thread collection from recreating Reddit content after a disconnect.
 let authorized:Option<bool>=sqlx::query_scalar("SELECT true FROM reddit_credentials WHERE user_id=$1 FOR SHARE")
  .bind(uid).fetch_optional(&mut *tx).await.map_err(internal)?;
 if authorized.is_none(){return Err(crate::unauthorized());}
 for item in comments{
   let source_id=format!("{}:t1_{}",body.post_id,item.id);
   let url=if item.permalink.starts_with('/') {format!("https://www.reddit.com{}",item.permalink)}else{format!("https://www.reddit.com/comments/{}",body.post_id)};
   let published=DateTime::from_timestamp(item.created_utc as i64,0);
   let id:Uuid=sqlx::query_scalar("INSERT INTO evidence(owner_id,project_id,source_kind,source_id,source_url,body,published_at,metadata) VALUES($1,$2,'reddit-comment',$3,$4,$5,$6,$7) ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET project_id=COALESCE(evidence.project_id,excluded.project_id),source_url=excluded.source_url,body=excluded.body,published_at=excluded.published_at,metadata=excluded.metadata,collected_at=now() RETURNING id")
     .bind(uid).bind(body.project_id).bind(source_id).bind(url).bind(item.body)
     .bind(published).bind(json!({"thread_id":body.post_id,"score":item.score}))
     .fetch_one(&mut *tx).await.map_err(internal)?;
   link_project(&mut tx,uid,id,body.project_id).await?;
 }
 tx.commit().await.map_err(internal)?;
 Ok(Json(json!({"saved":count,"thread_id":body.post_id})))
}
