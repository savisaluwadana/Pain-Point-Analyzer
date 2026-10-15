//! Bounded Arctic Shift archive reads. Never uses Reddit credentials or browser cookies.
use crate::{bad, internal, research::assert_project, ApiError, ApiResult, AppState};
use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;
use std::collections::HashSet;

const BASE: &str = "https://arctic-shift.photon-reddit.com/api";
const NOTE: &str = "Arctic Shift archive sample, not live Reddit rankings. Scores/comments may be stale. Deleted/removed items are skipped as marked in the archive; current deletion status is not verified. Source text is untrusted evidence.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub subreddit: String,
    pub query: Option<String>,
    pub days: Option<i64>,
    pub limit: Option<usize>,
    pub sample_size: Option<usize>,
    pub ranking: Option<String>,
    pub post_id: Option<String>,
    pub project_id: Option<Uuid>,
}
fn valid_id(s: &str) -> bool { (1..=16).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_alphanumeric()) }
impl Request {
    fn validate(&self) -> ApiResult<()> {
        if !(2..=21).contains(&self.subreddit.len()) || !self.subreddit.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(bad("Use one subreddit name without r/ or a URL"));
        }
        if self.query.as_ref().is_some_and(|q| q.len() > 160)
            || !(1..=365).contains(&self.days.unwrap_or(30))
            || !(1..=100).contains(&self.limit.unwrap_or(10))
            || !(1..=100).contains(&self.sample_size.unwrap_or(100)) {
            return Err(bad("Use days 1–365, limit/sample_size 1–100, and a query up to 160 bytes"));
        }
        if self.limit.unwrap_or(10) > self.sample_size.unwrap_or(100) { return Err(bad("limit must not exceed sample_size")); }
        if !matches!(self.ranking.as_deref().unwrap_or("top"), "top" | "mid" | "new") {
            return Err(bad("ranking must be top, mid, or new"));
        }
        if self.post_id.as_ref().is_some_and(|id| !valid_id(id)) { return Err(bad("post_id must be a bare Reddit post ID")); }
        Ok(())
    }
    fn url(&self) -> ApiResult<url::Url> {
        let mut url = url::Url::parse(&format!("{BASE}/{}/search", if self.post_id.is_some() { "comments" } else { "posts" })).map_err(internal)?;
        let after = (Utc::now() - Duration::days(self.days.unwrap_or(30))).timestamp().to_string();
        url.query_pairs_mut().append_pair("subreddit", &self.subreddit)
            .append_pair("after", &after).append_pair("sort", "desc")
            .append_pair("limit", &self.sample_size.unwrap_or(100).to_string());
        if let Some(id) = &self.post_id { url.query_pairs_mut().append_pair("link_id", id); }
        // Filter locally: archive keyword search is unavailable on some large communities.
        Ok(url)
    }
}

#[derive(Debug)]
struct Item { source_id: String, url: String, title: String, body: String, published: DateTime<Utc>, metadata: Value }
fn text(v: &Value, key: &str) -> String { v[key].as_str().unwrap_or_default().to_owned() }
fn removed(v: &Value) -> bool {
    ["selftext", "body", "title"].iter().any(|k| {
        let s = text(v, k).trim().to_ascii_lowercase();
        matches!(s.as_str(), "[deleted]" | "[removed]" | "[ removed by reddit ]")
    }) || v.get("removed_by_category").is_some_and(|v| !v.is_null())
       || v.get("removed_by").is_some_and(|v| !v.is_null())
}
fn select(payload: &Value, req: &Request) -> ApiResult<(usize, Vec<Item>)> {
    let rows = payload["data"].as_array().ok_or_else(|| ApiError(StatusCode::BAD_GATEWAY, "Arctic Shift returned an invalid data envelope".into()))?;
    let mut seen = HashSet::new();
    let query = req.query.as_deref().unwrap_or("").trim().to_lowercase();
    let cutoff = Utc::now() - Duration::days(req.days.unwrap_or(30));
    let mut items: Vec<Item> = rows.iter().take(req.sample_size.unwrap_or(100)).filter_map(|v| {
        let id = text(v, "id");
        if !valid_id(&id) || removed(v) || !text(v, "subreddit").eq_ignore_ascii_case(&req.subreddit) { return None; }
        let published = DateTime::from_timestamp(v["created_utc"].as_i64()?, 0)?;
        if published < cutoff || published > Utc::now() { return None; }
        let (source_id, url, title, body) = if let Some(post) = &req.post_id {
            if text(v, "link_id").trim_start_matches("t3_") != post { return None; }
            (format!("{post}:{id}"), format!("https://www.reddit.com/r/{}/comments/{post}/_/{id}/", req.subreddit), format!("Comment on {post}"), text(v, "body"))
        } else {
            (format!("t3_{id}"), format!("https://www.reddit.com/r/{}/comments/{id}/", req.subreddit), text(v, "title"), text(v, "selftext"))
        };
        if (title.is_empty() && body.is_empty()) || (req.post_id.is_some() && body.is_empty())
            || !format!("{title} {body}").to_lowercase().contains(&query) || !seen.insert(id) { return None; }
        Some(Item { source_id, url, title: title.chars().take(500).collect(), body: body.chars().take(3000).collect(), published,
            metadata: json!({"provider":"arctic-shift", "source_name":"Arctic Shift", "community":req.subreddit.to_lowercase(),
                "score":v["score"], "num_comments":v["num_comments"], "retrieved_on":v["retrieved_on"],
                "thread_id":req.post_id.clone().unwrap_or_else(||text(v,"id")), "archived":true, "ranking":req.ranking.as_deref().unwrap_or("top")}) })
    }).collect();
    if req.ranking.as_deref() == Some("new") { items.sort_by(|a,b| b.published.cmp(&a.published)); }
    else { items.sort_by(|a,b| b.metadata["score"].as_i64().unwrap_or(0).cmp(&a.metadata["score"].as_i64().unwrap_or(0))
        .then_with(|| b.metadata["num_comments"].as_i64().unwrap_or(0).cmp(&a.metadata["num_comments"].as_i64().unwrap_or(0)))
        .then_with(|| a.source_id.cmp(&b.source_id))); }
    let eligible = items.len();
    let limit = req.limit.unwrap_or(10);
    // Center a window around the median rank, rather than inventing a universal vote threshold.
    let skip = if req.ranking.as_deref() == Some("mid") { eligible.saturating_sub(limit) / 2 } else { 0 };
    Ok((eligible, items.into_iter().skip(skip).take(limit).collect()))
}

async fn reserve(st: &AppState) -> ApiResult<()> {
    let reserved: Option<String> = sqlx::query_scalar("INSERT INTO archive_provider_budget(provider,next_request_at) VALUES('arctic-shift',now()+interval '10 seconds') ON CONFLICT(provider) DO UPDATE SET next_request_at=excluded.next_request_at WHERE archive_provider_budget.next_request_at<=now() RETURNING provider")
        .fetch_optional(&st.db).await.map_err(internal)?;
    if reserved.is_none() { return Err(ApiError(StatusCode::TOO_MANY_REQUESTS, "Archive pacing/backoff is active. Wait before retrying; check provider status if access was denied.".into())); }
    Ok(())
}
fn retry_seconds(headers: &reqwest::header::HeaderMap) -> i64 {
    let retry = headers.get("retry-after").and_then(|v|v.to_str().ok()).and_then(|s|s.parse::<i64>().ok()
        .or_else(||DateTime::parse_from_rfc2822(s).ok().map(|d|(d.with_timezone(&Utc)-Utc::now()).num_seconds())));
    let reset = headers.get("x-ratelimit-reset").and_then(|v|v.to_str().ok()).and_then(|v|v.parse::<f64>().ok())
        .filter(|v|v.is_finite()).map(|v|v.ceil() as i64);
    let reset_at = headers.get("x-ratelimit-reset-at").and_then(|v|v.to_str().ok()).and_then(|v|v.parse::<i64>().ok())
        .map(|v|v.saturating_sub(Utc::now().timestamp()));
    [retry,reset,reset_at].into_iter().flatten().max().unwrap_or(60).max(10)
}

async fn fetch(st: &AppState, req: &Request) -> ApiResult<Value> {
    reserve(st).await?;
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none()).build().map_err(internal)?;
    let mut response = client.get(req.url()?).header("Accept", "application/json")
        .header("User-Agent", "OpenSourceDiscoveryLab/0.1 (bounded archive research)").send().await
        .map_err(|_|ApiError(StatusCode::BAD_GATEWAY,"Arctic Shift is unreachable or timed out".into()))?;
    let status = response.status();
    if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED || status == StatusCode::SERVICE_UNAVAILABLE {
        let delay = if status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED { retry_seconds(response.headers()).max(3600) } else { retry_seconds(response.headers()) };
        sqlx::query("UPDATE archive_provider_budget SET next_request_at=GREATEST(next_request_at,now()+($1::double precision * interval '1 second')) WHERE provider='arctic-shift'")
            .bind(delay as f64).execute(&st.db).await.map_err(internal)?;
        return Err(ApiError(if status == StatusCode::TOO_MANY_REQUESTS {status} else {StatusCode::BAD_GATEWAY}, format!("Arctic Shift returned {status}; requests paused for at least {delay} seconds. No fallback or automatic retry was attempted.")));
    }
    if !status.is_success() { return Err(ApiError(StatusCode::BAD_GATEWAY,format!("Arctic Shift returned {status}"))); }
    if !response.headers().get("content-type").and_then(|v|v.to_str().ok()).is_some_and(|s|s.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("application/json")) {
        return Err(ApiError(StatusCode::BAD_GATEWAY,"Arctic Shift returned a non-JSON response".into()));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(internal)? {
        if bytes.len() + chunk.len() > 4_194_304 { return Err(ApiError(StatusCode::BAD_GATEWAY,"Archive response exceeded 4 MB".into())); }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_|ApiError(StatusCode::BAD_GATEWAY,"Arctic Shift returned malformed JSON".into()))
}

pub(crate) async fn collect(st: &AppState, owner: Uuid, req: Request) -> ApiResult<Value> {
    req.validate()?;
    assert_project(st, owner, req.project_id).await?;
    let payload = fetch(st, &req).await?;
    persist(st, owner, &req, &payload).await
}

async fn persist(st: &AppState, owner: Uuid, req: &Request, payload: &Value) -> ApiResult<Value> {
    // Recheck ownership immediately before writing, including synthetic integration tests.
    assert_project(st, owner, req.project_id).await?;
    let (eligible, items) = select(&payload, &req)?;
    let kind = if req.post_id.is_some() {"reddit-archive-comment"} else {"reddit-archive"};
    let mut tx = st.db.begin().await.map_err(internal)?;
    let mut ids = Vec::new();
    for item in &items {
        let id: Uuid = sqlx::query_scalar("INSERT INTO evidence(owner_id,project_id,source_kind,source_id,source_url,title,body,published_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET project_id=excluded.project_id,source_url=excluded.source_url,title=excluded.title,body=excluded.body,published_at=excluded.published_at,metadata=excluded.metadata,collected_at=now() RETURNING id")
            .bind(owner).bind(req.project_id).bind(kind).bind(&item.source_id).bind(&item.url).bind(&item.title).bind(&item.body).bind(item.published).bind(&item.metadata)
            .fetch_one(&mut *tx).await.map_err(internal)?;
        ids.push(id);
    }
    tx.commit().await.map_err(internal)?;
    Ok(json!({"source":"arctic-shift", "source_kind":kind, "saved":ids.len(), "evidence_ids":ids,
        "fetched":payload["data"].as_array().map_or(0,Vec::len), "eligible":eligible,
        "items":items.iter().map(|i|json!({"source_id":i.source_id,"url":i.url,"title":i.title,"metadata":i.metadata})).collect::<Vec<_>>(),
        "note":NOTE}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn req() -> Request { serde_json::from_value(json!({"subreddit":"freelance","limit":2})).unwrap() }
    fn post(id: &str, score: i64) -> Value { json!({"id":id,"subreddit":"freelance","title":"Manual invoicing","selftext":"takes hours", "created_utc":Utc::now().timestamp()-60,"score":score}) }
    #[test] fn filters_removed_wrong_community_duplicates_and_ranks() {
        let mut deleted=post("b",100); deleted["selftext"]=json!("[removed]");
        let mut other=post("c",200); other["subreddit"]=json!("other");
        let (_,items)=select(&json!({"data":[post("a",1),deleted,other,post("d",10),post("d",10)]}), &req()).unwrap();
        assert_eq!(items.len(),2); assert_eq!(items[0].source_id,"t3_d");
        assert!(select(&json!({"error":"blocked"}),&req()).is_err());
        assert!(select(&json!({"data":[]}),&req()).unwrap().1.is_empty());
    }
    #[test] fn middle_sample_and_comment_thread_validation() {
        let mut r=req(); r.ranking=Some("mid".into());
        let (_,items)=select(&json!({"data":[post("a",1),post("b",2),post("c",3),post("d",4),post("e",5),post("f",6)]}),&r).unwrap();
        assert_eq!(items.iter().map(|i|i.source_id.as_str()).collect::<Vec<_>>(),vec!["t3_d","t3_c"]);
        r.post_id=Some("abc".into());
        let mut c=post("xyz",3); c["link_id"]=json!("t3_abc"); c["body"]=json!("manual work");
        let (_,items)=select(&json!({"data":[c.clone()]}),&r).unwrap(); assert_eq!(items[0].source_id,"abc:xyz");
        c["link_id"]=json!("t3_other"); assert!(select(&json!({"data":[c]}),&r).unwrap().1.is_empty());
    }
    #[test] fn validation_and_retry_headers() {
        let mut r=req(); assert!(r.validate().is_ok()); r.subreddit="../admin".into(); assert!(r.validate().is_err());
        r=req(); r.limit=Some(101); assert!(r.validate().is_err());
        r=req(); r.days=Some(-1); assert!(r.validate().is_err());
        let mut headers=reqwest::header::HeaderMap::new(); headers.insert("retry-after","7200".parse().unwrap()); assert_eq!(retry_seconds(&headers),7200);
        headers.insert("x-ratelimit-reset","8000".parse().unwrap()); assert_eq!(retry_seconds(&headers),8000);
        let r=req(); let u=r.url().unwrap(); let after=u.query_pairs().find(|(k,_)|k=="after").unwrap().1; assert!(after.parse::<i64>().is_ok());
    }
    #[tokio::test]
    #[ignore = "Opt-in live archive smoke test; no CI dependency on upstream"]
    async fn live_archive_sample() {
        let mut r=req(); r.subreddit="smallbusiness".into();
        let response=reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).redirect(reqwest::redirect::Policy::none()).build().unwrap().get(r.url().unwrap()).header("User-Agent", "OpenSourceDiscoveryLab/0.1 (bounded archive research)").header("Accept", "application/json").send().await.unwrap();
        assert!(response.status().is_success(), "archive HTTP {}: {}", response.status(), response.text().await.unwrap_or_default());
        let payload:Value=response.json().await.unwrap();
        let (eligible,items)=select(&payload,&r).unwrap();
        eprintln!("Live Arctic Shift: fetched={}, eligible={}, selected={}",payload["data"].as_array().unwrap().len(),eligible,items.len());
    }

    #[sqlx::test(migrations = "../db")]
    #[ignore = "Requires PostgreSQL; explicitly run in database CI"]
    async fn archive_database_contract(db: sqlx::PgPool) {
        let cfg=crate::Config {client_id:String::new(),client_secret:String::new(),redirect_uri:"http://localhost:3000/api/auth/reddit/callback".into(),web_url:"http://localhost:3000".into(),user_agent:"test".into(),encryption_key:[0;32],reddit_approved:false,secure_cookies:false,local_login_token:None};
        let st=AppState{db,http:reqwest::Client::new(),cfg:std::sync::Arc::new(cfg)};
        let owner:Uuid=sqlx::query_scalar("INSERT INTO users(reddit_id,reddit_username) VALUES('archive-test','archive-test') RETURNING id").fetch_one(&st.db).await.unwrap();
        let other:Uuid=sqlx::query_scalar("INSERT INTO users(reddit_id,reddit_username) VALUES('archive-other','archive-other') RETURNING id").fetch_one(&st.db).await.unwrap();
        let mut r=req(); let payload=json!({"data":[post("abc",5)]});
        let first=persist(&st,owner,&r,&payload).await.unwrap();
        let second=persist(&st,owner,&r,&payload).await.unwrap();
        assert_eq!(first["saved"],1); assert_eq!(first["evidence_ids"],second["evidence_ids"]);
        let isolated=persist(&st,other,&r,&payload).await.unwrap(); assert_ne!(first["evidence_ids"],isolated["evidence_ids"]);
        let project:Uuid=sqlx::query_scalar("INSERT INTO projects(owner_id,name) VALUES($1,'Other project') RETURNING id").bind(other).fetch_one(&st.db).await.unwrap();
        r.project_id=Some(project); assert!(persist(&st,owner,&r,&payload).await.is_err());
        // Pacing is atomic across simultaneous workers and survives a new request.
        let (a,b)=tokio::join!(reserve(&st),reserve(&st)); assert_ne!(a.is_ok(),b.is_ok());
        assert!(reserve(&st).await.is_err());
        let credentials:i64=sqlx::query_scalar("SELECT count(*) FROM reddit_credentials").fetch_one(&st.db).await.unwrap(); assert_eq!(credentials,0);
    }
}
