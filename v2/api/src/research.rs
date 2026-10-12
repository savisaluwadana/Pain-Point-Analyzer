use crate::{oauth,reddit,scoring::{self,EvidenceText},AppState,ApiResult,bad,internal};
use axum::{extract::{Query,State},http::{HeaderMap,header::{SET_COOKIE},HeaderValue},response::{IntoResponse,Response},Json};
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
#[derive(Deserialize)]
pub struct EvidenceInput {source_kind:String,source_id:String,source_url:String,title:Option<String>,body:Option<String>,project_id:Option<Uuid>}
#[derive(Serialize,FromRow)]
pub struct EvidenceRow {id:Uuid,source_kind:String,source_id:String,source_url:String,title:String,body:String,collected_at:DateTime<Utc>}
#[derive(Deserialize)]
pub struct EvidenceQuery {limit:Option<i64>,project_id:Option<Uuid>}
pub async fn evidence(State(st):State<AppState>,headers:HeaderMap,Query(q):Query<EvidenceQuery>)->ApiResult<Json<Vec<EvidenceRow>>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let rows=sqlx::query_as("SELECT id,source_kind,source_id,source_url,title,body,collected_at FROM evidence WHERE owner_id=$1 AND ($2::uuid IS NULL OR project_id=$2) ORDER BY collected_at DESC LIMIT $3")
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
pub async fn ingest_evidence(State(st):State<AppState>,headers:HeaderMap,Json(body):Json<EvidenceInput>)->ApiResult<Json<Value>>{
 let uid=oauth::user_id(&st,&headers).await?;
 assert_project(&st,uid,body.project_id).await?;
 if body.source_kind!="manual"&&body.source_kind!="import"{return Err(bad("Use the authorized Reddit connector for Reddit evidence"));}
 if body.source_id.len()>255||body.source_id.is_empty()||body.source_url.len()>2048{return Err(bad("Invalid evidence identifier or URL"));}
 let title=body.title.unwrap_or_default().chars().take(500).collect::<String>();
 let text=body.body.unwrap_or_default().chars().take(10000).collect::<String>();
 let id:Uuid=sqlx::query_scalar("INSERT INTO evidence(owner_id,project_id,source_kind,source_id,source_url,title,body) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET title=excluded.title,body=excluded.body,collected_at=now() RETURNING id")
  .bind(uid).bind(body.project_id).bind(&body.source_kind).bind(&body.source_id).bind(&body.source_url).bind(title).bind(text).fetch_one(&st.db).await.map_err(internal)?;
 Ok(Json(json!({"id":id})))
}
#[derive(Deserialize)]
pub struct ScanInput {subreddit:String,project_id:Option<Uuid>,limit:Option<i32>}
pub(crate) async fn persist_posts(st:&AppState,uid:Uuid,posts:Vec<reddit::RedditPost>,project:Option<Uuid>)->ApiResult<usize>{
 let mut tx=st.db.begin().await.map_err(internal)?;
 let count=posts.len();
 for post in posts{
   let url=format!("https://www.reddit.com{}",post.permalink);
   let published=DateTime::from_timestamp(post.created_utc as i64,0);
   sqlx::query("INSERT INTO evidence(owner_id,project_id,source_kind,source_id,source_url,title,body,author,published_at,metadata) VALUES($1,$2,'reddit',$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET title=excluded.title,body=excluded.body,author=excluded.author,metadata=excluded.metadata,collected_at=now()")
    .bind(uid).bind(project).bind(format!("t3_{}",post.id)).bind(url)
    .bind(post.title.chars().take(500).collect::<String>()).bind(post.selftext.chars().take(10000).collect::<String>())
    .bind(None::<String>).bind(published).bind(json!({"subreddit":post.subreddit,"score":post.score,"num_comments":post.num_comments}))
    .execute(&mut *tx).await.map_err(internal)?;
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
 let rows:Vec<(Uuid,String,String,String,String)>=sqlx::query_as("SELECT id,source_kind,source_id,title,body FROM evidence WHERE owner_id=$1 AND ($2::uuid IS NULL OR project_id=$2) ORDER BY collected_at DESC LIMIT 1000")
   .bind(uid).bind(body.project_id).fetch_all(&st.db).await.map_err(internal)?;
 let items=rows.into_iter().map(|(id,source_kind,source_id,title,body)|EvidenceText{id,source_kind,source_id,text:format!("{title}\n{body}")}).collect::<Vec<_>>();
 let report=scoring::analyze(&items);
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
pub struct RunRow {id:Uuid,topic:String,report:Value,created_at:DateTime<Utc>}
pub async fn runs(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Json<Vec<RunRow>>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let rows=sqlx::query_as("SELECT id,topic,report,created_at FROM research_runs WHERE owner_id=$1 ORDER BY created_at DESC LIMIT 100").bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 Ok(Json(rows))
}
#[derive(Serialize,FromRow)]
pub struct OpportunityRow {id:Uuid,title:String,problem:String,score:f64,evidence_refs:Value,created_at:DateTime<Utc>}
pub async fn opportunities(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Json<Vec<OpportunityRow>>>{
 let uid=oauth::user_id(&st,&headers).await?;
 let rows=sqlx::query_as("SELECT id,title,problem,score,evidence_refs,created_at FROM opportunities WHERE owner_id=$1 ORDER BY score DESC,created_at DESC LIMIT 100").bind(uid).fetch_all(&st.db).await.map_err(internal)?;
 Ok(Json(rows))
}
pub async fn delete_data(State(st):State<AppState>,headers:HeaderMap)->ApiResult<Response>{
 let uid=oauth::user_id(&st,&headers).await?;
 sqlx::query("DELETE FROM users WHERE id=$1").bind(uid).execute(&st.db).await.map_err(internal)?;
 let mut response=Json(json!({"deleted":true,"action":"Your stored account, credentials, evidence, research and sessions were deleted"})).into_response();
 response.headers_mut().insert(SET_COOKIE,HeaderValue::from_static("pil_session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"));
 Ok(response)
}
