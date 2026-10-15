//! Read-only, bounded connectors for approved public sources.
//! No arbitrary URL fetches: upstream hosts are fixed or explicitly allowlisted.
use crate::{oauth,research::assert_project,AppState,ApiError,ApiResult,bad,internal};
use axum::{extract::State,http::{HeaderMap,StatusCode},Json};
use chrono::{DateTime,Utc};
use reqwest::{Client,StatusCode as UpstreamStatus};
use serde::Deserialize;
use serde_json::{json,Value};
use sqlx::types::Uuid;
use std::env;
use url::Url;

#[derive(Deserialize)]
pub struct CollectInput {
    source:String,
    query:Option<String>,
    target:Option<String>,
    project_id:Option<Uuid>,
    limit:Option<usize>,
    days:Option<i64>,
    ranking:Option<String>,
    sample_size:Option<usize>,
    post_id:Option<String>,
}
#[derive(Clone,Debug)]
struct Finding {
    id:String,
    title:String,
    body:String,
    url:String,
    published:Option<DateTime<Utc>>,
    metadata:Value,
}
fn short(s:&str,max:usize)->String{s.chars().take(max).collect()}
fn raw(v:&Value,path:&str)->String{v.pointer(path).and_then(Value::as_str).unwrap_or("").to_owned()}
fn get_date(s:&str)->Option<DateTime<Utc>>{
    DateTime::parse_from_rfc3339(s).ok().map(|x|x.with_timezone(&Utc))
}
fn html_text(input:&str)->String{
    // Deliberately store a short plain-text excerpt rather than HTML or full articles.
    let mut out=String::new();
    let mut tag=false;
    for c in input.chars(){
        if c=='<' { tag=true;out.push(' '); }
        else if c=='>' {tag=false;}
        else if !tag {out.push(c);}
        if out.len()>6000 {break;}
    }
    short(&out.replace("&amp;","&").replace("&lt;","<").replace("&gt;",">")
        .replace("&quot;","\"").replace("&#39;","'").split_whitespace().collect::<Vec<_>>().join(" "),1200)
}
fn supported(source:&str)->bool{matches!(source,"github"|"hackernews"|"stackoverflow"|"bluesky"|"youtube"|"discourse"|"medium")}
fn required_env(key:&str,help:&str)->ApiResult<String>{
    env::var(key).ok().filter(|s|!s.trim().is_empty())
        .ok_or_else(||ApiError(StatusCode::SERVICE_UNAVAILABLE,help.to_owned()))
}
fn medium_tag(tag:&str)->ApiResult<&str>{
    if !(2..=60).contains(&tag.len())||!tag.bytes().all(|x|x.is_ascii_lowercase()||x.is_ascii_digit()||x==b'-'){
        return Err(bad("Medium topic must be a lowercase tag such as 'open-source'"));
    }
    Ok(tag)
}
fn youtube_id(id:&str)->ApiResult<&str>{
    if id.len()!=11||!id.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'_'||c==b'-'){return Err(bad("YouTube target must be an 11-character video ID"));}
    Ok(id)
}
fn discourse_host(target:&str)->ApiResult<String>{
    let host=target.to_ascii_lowercase();
    if host.len()>200 || !host.contains('.') || !host.bytes().all(|b|b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'.'||b==b'-')
       || host.starts_with('-') || host.ends_with('.') || host.contains(".."){
        return Err(bad("Discourse target must be an allowlisted public hostname"));
    }
    // Never permit loopback, private IPs, or internal test domains even if misconfigured.
    if host.parse::<std::net::IpAddr>().is_ok()
        || host=="localhost" || host.ends_with(".localhost")
        || host.ends_with(".local") || host.ends_with(".internal")
        || host.ends_with(".test") || host.ends_with(".invalid"){
        return Err(bad("Discourse hostname must be a public DNS name"));
    }
    // Site owner/operator must configure the exact public forums they have permission to query.
    let allowed=env::var("DISCOURSE_ALLOWED_HOSTS").unwrap_or_default();
    if !allowed.split(',').map(str::trim).any(|v|v.eq_ignore_ascii_case(&host)){
        return Err(bad("Discourse hostname is not in DISCOURSE_ALLOWED_HOSTS"));
    }
    Ok(host)
}
fn status_error(status:UpstreamStatus,source:&str)->ApiError{
    if status==UpstreamStatus::TOO_MANY_REQUESTS || status==UpstreamStatus::FORBIDDEN{
        ApiError(StatusCode::TOO_MANY_REQUESTS,format!("{source} access was refused or rate limited; check credentials, permissions, and provider limits"))
    }else if status==UpstreamStatus::UNAUTHORIZED {
        ApiError(StatusCode::BAD_GATEWAY,format!("{source} authentication was rejected"))
    }else{
        ApiError(StatusCode::BAD_GATEWAY,format!("{source} returned HTTP {status}"))
    }
}
async fn fetch_bytes(client:&Client,source:&str,url:Url,auth:Option<(&str,String)>)->ApiResult<Vec<u8>>{
    let mut req=client.get(url).header("Accept","application/json, application/rss+xml;q=0.9, */*;q=0.5")
        .header("User-Agent","OpenSourceDiscoveryLab/0.1 (+https://github.com/savisaluwadana/Reddit-post-Analyzer)");
    if let Some((name,token))=auth{req=req.header(name,token);}
    // Redirects are disabled for this client to prevent allowlist bypass / cross-host token disclosure.
    let mut response=req.send().await.map_err(|_|ApiError(StatusCode::BAD_GATEWAY,format!("{source} is currently unreachable")))?;
    if !response.status().is_success(){return Err(status_error(response.status(),source));}
    if response.content_length().is_some_and(|len|len>1_048_576){
        return Err(ApiError(StatusCode::BAD_GATEWAY,"Upstream response was too large".into()));
    }
    let mut result=Vec::new();
    while let Some(chunk)=response.chunk().await.map_err(internal)?{
        if result.len()+chunk.len()>1_048_576{return Err(ApiError(StatusCode::BAD_GATEWAY,"Upstream response exceeded 1 MB".into()));}
        result.extend_from_slice(&chunk);
    }
    Ok(result)
}
async fn get_json(client:&Client,source:&str,url:Url,auth:Option<(&str,String)>)->ApiResult<Value>{
    let bytes=fetch_bytes(client,source,url,auth).await?;
    serde_json::from_slice(&bytes).map_err(|_|ApiError(StatusCode::BAD_GATEWAY,format!("{source} returned malformed JSON")))
}
fn uri(base:&str,pairs:&[(&str,&str)])->ApiResult<Url>{
    let mut url=Url::parse(base).map_err(internal)?;
    url.query_pairs_mut().extend_pairs(pairs.iter().copied());
    Ok(url)
}
/// Authors can ask to be hidden from logged-out viewers; collection is logged out, so skip them.
fn bluesky_opted_out(post:&Value)->bool{
    post.pointer("/author/labels").and_then(Value::as_array)
        .is_some_and(|labels|labels.iter().any(|l|l["val"]=="!no-unauthenticated"))
}
fn bluesky_findings(v:&Value)->Vec<Finding>{
    v["posts"].as_array().map(|a|a.iter().filter_map(|x|{
        if bluesky_opted_out(x){return None;}
        let id=raw(x,"/uri");let handle=raw(x,"/author/handle");
        let rkey=id.rsplit('/').next()?.to_owned();
        if !id.starts_with("at://")||!handle.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'.'||c==b'-')
            || !rkey.bytes().all(|c|c.is_ascii_alphanumeric()){return None;}
        Some(Finding{id,title:format!("Bluesky post by @{handle}"),body:short(&raw(x,"/record/text"),1200),
            url:format!("https://bsky.app/profile/{handle}/post/{rkey}"),
            published:get_date(&raw(x,"/record/createdAt")),
            metadata:json!({"indexed_at":x["indexedAt"],"source":"Bluesky AppView (api.bsky.app)"})})
    }).collect()).unwrap_or_default()
}
async fn collect_upstream(source:&str,q:&str,target:&str,limit:usize)->ApiResult<Vec<Finding>>{
    let client=Client::builder().timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none()).build().map_err(internal)?;
    let n=limit.to_string();
    let data:Vec<Finding>=match source{
        "github"=>{
            let phrase=format!("is:issue {q}");
            let u=uri("https://api.github.com/search/issues",&[("q",&phrase),("per_page",&n)])?;
            let auth=env::var("GITHUB_SOURCE_TOKEN").ok().filter(|x|!x.is_empty()).map(|x|("Authorization",format!("Bearer {x}")));
            let v=get_json(&client,source,u,auth).await?;
            v["items"].as_array().map(|a|a.iter().filter_map(|x|{
                let url=raw(x,"/html_url");if !url.starts_with("https://github.com/"){return None;}
                Some(Finding{id:raw(x,"/node_id"),title:raw(x,"/title"),body:short(&raw(x,"/body"),1200),url,
                published:get_date(&raw(x,"/created_at")),
                metadata:json!({"repository":raw(x,"/repository_url"),"comments":x["comments"],"state":x["state"],"source":"GitHub public Issues"})})
            }).collect()).unwrap_or_default()
        },
        "hackernews"=>{
            // Community-maintained Algolia full-text index of public Hacker News data.
            let u=uri("https://hn.algolia.com/api/v1/search_by_date",&[("query",q),("tags","comment"),("hitsPerPage",&n)])?;
            let v=get_json(&client,source,u,None).await?;
            v["hits"].as_array().map(|a|a.iter().filter_map(|x|{
                let id=raw(x,"/objectID");if id.is_empty()||!id.bytes().all(|b|b.is_ascii_digit()){return None;}
                Some(Finding{id:id.clone(),title:format!("HN discussion on {q}"),
                body:html_text(&raw(x,"/comment_text")),
                url:format!("https://news.ycombinator.com/item?id={id}"),
                published:get_date(&raw(x,"/created_at")),
                metadata:json!({"story_id":x["story_id"],"index":"Algolia HN Search (third-party)"})})
            }).collect()).unwrap_or_default()
        },
        "stackoverflow"=>{
            let u=uri("https://api.stackexchange.com/2.3/search/advanced",&[("q",q),("site","stackoverflow"),("pagesize",&n),("sort","relevance"),("order","desc"),("filter","withbody")])?;
            let v=get_json(&client,source,u,None).await?;
            if v["backoff"].as_u64().is_some(){return Err(ApiError(StatusCode::TOO_MANY_REQUESTS,"Stack Exchange requires a backoff before making another request".into()));}
            v["items"].as_array().map(|a|a.iter().filter_map(|x|{
                let url=raw(x,"/link");let id=x["question_id"].as_i64()?;
                if !url.starts_with("https://stackoverflow.com/"){return None;}
                Some(Finding{id:id.to_string(),title:html_text(&raw(x,"/title")),body:html_text(&raw(x,"/body")),
                url,published:x["creation_date"].as_i64().and_then(|t|DateTime::from_timestamp(t,0)),
                metadata:json!({"score":x["score"],"tags":x["tags"],"author":raw(x,"/owner/display_name"),"license":"CC BY-SA; version depends on post date","license_url":"https://stackoverflow.com/help/licensing"})})
            }).collect()).unwrap_or_default()
        },
        "bluesky"=>{
            // public.api.bsky.app is a cached mirror that answers searchPosts with 403;
            // api.bsky.app serves unauthenticated search.
            let u=uri("https://api.bsky.app/xrpc/app.bsky.feed.searchPosts",&[("q",q),("limit",&n)])?;
            let v=get_json(&client,source,u,None).await?;
            bluesky_findings(&v)
        },
        "youtube"=>{
            let id=youtube_id(target)?;
            let key=required_env("YOUTUBE_API_KEY","YouTube collection requires YOUTUBE_API_KEY")?;
            let mut args=vec![("part","snippet"),("videoId",id),("maxResults",&n),("textFormat","plainText"),("key",key.as_str())];
            if !q.is_empty(){args.push(("searchTerms",q));}
            let u=uri("https://www.googleapis.com/youtube/v3/commentThreads",&args)?;
            let v=get_json(&client,source,u,None).await?;
            v["items"].as_array().map(|a|a.iter().filter_map(|x|{
                let comment_id=raw(x,"/snippet/topLevelComment/id");
                if comment_id.is_empty()||!comment_id.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'-'||c==b'_'){return None;}
                let comment=&x["snippet"]["topLevelComment"]["snippet"];
                Some(Finding{id:comment_id.clone(),title:format!("YouTube comment on {id}"),body:short(&raw(comment,"/textOriginal"),1200),
                url:format!("https://www.youtube.com/watch?v={id}&lc={comment_id}"),
                published:get_date(&raw(comment,"/publishedAt")),
                metadata:json!({"video_id":id,"like_count":comment["likeCount"],"source":"YouTube Data API; provider policies apply"})})
            }).collect()).unwrap_or_default()
        },
        "discourse"=>{
            let host=discourse_host(target)?;
            let u=uri(&format!("https://{host}/search.json"),&[("q",q)])?;
            let v=get_json(&client,source,u,None).await?;
            v["posts"].as_array().map(|a|a.iter().take(limit).filter_map(|x|{
                let post_id=x["id"].as_i64()?;
                let topic_id=x["topic_id"].as_i64()?;
                let post_number=x["post_number"].as_i64().unwrap_or(1);
                if post_id<=0||topic_id<=0||post_number<=0{return None;}
                Some(Finding{id:format!("{host}:{post_id}"),title:format!("Discourse discussion #{topic_id}"),
                body:html_text(&raw(x,"/blurb")),
                url:format!("https://{host}/t/{topic_id}/{post_number}"),
                published:get_date(&raw(x,"/created_at")),
                metadata:json!({"forum":host,"topic_id":topic_id,"post_number":post_number,"public_search":true})})
            }).collect()).unwrap_or_default()
        },
        "medium"=>{
            let tag=medium_tag(q)?;
            let u=Url::parse(&format!("https://medium.com/feed/tag/{tag}")).map_err(internal)?;
            let body=fetch_bytes(&client,source,u,None).await?;
            let channel=rss::Channel::read_from(&body[..]).map_err(|_|ApiError(StatusCode::BAD_GATEWAY,"Medium returned an invalid RSS feed".into()))?;
            channel.items().iter().take(limit).filter_map(|x|{
                let url=x.link()?.to_string();
                if !url.starts_with("https://medium.com/"){return None;}
                Some(Finding{id:x.guid().map(|g|g.value()).unwrap_or(&url).to_owned(),
                title:x.title().unwrap_or("Medium article").to_owned(),
                body:html_text(x.description().unwrap_or("")),
                url,published:x.pub_date().and_then(|d|DateTime::parse_from_rfc2822(d).ok()).map(|d|d.with_timezone(&Utc)),
                metadata:json!({"topic_tag":tag,"discovery":"Medium public RSS; excerpts only, no paywall bypass"})})
            }).collect()
        },
        _=>return Err(bad("Unsupported source"))
    };
    Ok(data.into_iter().filter(|v|!v.id.is_empty() && !v.url.is_empty()).take(limit).collect())
}
async fn budget(st:&AppState,uid:Uuid,source:&str)->ApiResult<()>{
    let mut tx=st.db.begin().await.map_err(internal)?;
    sqlx::query("INSERT INTO source_request_budgets(owner_id,source) VALUES($1,$2) ON CONFLICT DO NOTHING")
        .bind(uid).bind(source).execute(&mut *tx).await.map_err(internal)?;
    let (started,count):(DateTime<Utc>,i32)=sqlx::query_as(
        "SELECT window_started_at,request_count FROM source_request_budgets WHERE owner_id=$1 AND source=$2 FOR UPDATE")
        .bind(uid).bind(source).fetch_one(&mut *tx).await.map_err(internal)?;
    let reset=Utc::now().signed_duration_since(started).num_seconds()>=60;
    if !reset&&count>=4{return Err(ApiError(StatusCode::TOO_MANY_REQUESTS,"Collection is limited to four requests per source per minute; try later".into()));}
    sqlx::query("UPDATE source_request_budgets SET window_started_at=CASE WHEN $3 THEN now() ELSE window_started_at END,request_count=$4 WHERE owner_id=$1 AND source=$2")
        .bind(uid).bind(source).bind(reset).bind(if reset{1}else{count+1}).execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(())
}
pub async fn catalog()->Json<Value>{
    let youtube=env::var("YOUTUBE_API_KEY").is_ok_and(|v|!v.trim().is_empty());
    let discourse=env::var("DISCOURSE_ALLOWED_HOSTS").is_ok_and(|v|!v.trim().is_empty());
    Json(json!({"sources":[
        {"id":"arctic-shift","name":"Reddit archive (Arctic Shift)","mode":"subreddit","ready":true,"note":"No Reddit login/key. Archived samples, not live rankings. One subreddit per call; pace calls at least 10 seconds apart. Review data-use terms."},
        {"id":"github","name":"GitHub Issues","mode":"keyword","ready":true,"note":"Read-only public issues; rate-limited. Optional GitHub token."},
        {"id":"hackernews","name":"Hacker News","mode":"keyword","ready":true,"note":"Public comments via third-party Algolia HN index."},
        {"id":"stackoverflow","name":"Stack Overflow","mode":"keyword","ready":true,"note":"Stack Exchange API; respect CC BY-SA attribution and backoff."},
        {"id":"bluesky","name":"Bluesky","mode":"keyword","ready":true,"note":"Unauthenticated AppView search (api.bsky.app); skips authors who opt out of logged-out visibility."},
        {"id":"youtube","name":"YouTube Comments","mode":"video","ready":youtube,"note":"Requires YouTube Data API key and a video ID; subject to quota."},
        {"id":"discourse","name":"Discourse Forums","mode":"forum","ready":discourse,"note":"Public search on explicitly allowlisted community domains."},
        {"id":"medium","name":"Medium RSS","mode":"tag","ready":true,"note":"Public topic RSS; article excerpts only, no paywalled content."}
    ]}))
}
pub async fn collect(State(st):State<AppState>,headers:HeaderMap,Json(req):Json<CollectInput>)->ApiResult<Json<Value>>{
    let user=oauth::user_id(&st,&headers).await?;
    assert_project(&st,user,req.project_id).await?;
    if req.source=="arctic-shift" {
        return Ok(Json(crate::archive::collect(&st,user,crate::archive::Request {
            subreddit:req.target.unwrap_or_default().trim().to_owned(),query:req.query,days:req.days,
            limit:req.limit,sample_size:req.sample_size,ranking:req.ranking,post_id:req.post_id,project_id:req.project_id,
        }).await?));
    }
    if !supported(&req.source){return Err(bad("Unsupported source"));}
    let q=req.query.as_deref().unwrap_or("").trim();
    let target=req.target.as_deref().unwrap_or("").trim();
    if q.len()>160||target.len()>220{return Err(bad("Search term or target is too long"));}
    if req.source!="youtube" && q.len()<2{return Err(bad("Enter a search term or topic tag"));}
    let limit=req.limit.unwrap_or(15).clamp(1,25);
    // Check all request-specific validation BEFORE charging a local API budget.
    match req.source.as_str(){
        "youtube"=>{youtube_id(target)?;required_env("YOUTUBE_API_KEY","Set YOUTUBE_API_KEY to collect YouTube comments")?;},
        "discourse"=>{discourse_host(target)?;},
        "medium"=>{medium_tag(q)?;},
        _=>{}
    };
    budget(&st,user,&req.source).await?;
    let items=collect_upstream(&req.source,q,target,limit).await?;
    let mut tx=st.db.begin().await.map_err(internal)?;
    let mut saved=0usize;
    for finding in items.iter(){
        let meta=json!({"connector":req.source,"collected_via":"official-or-published-public-interface","details":finding.metadata});
        sqlx::query("INSERT INTO evidence(owner_id,project_id,source_kind,source_id,source_url,title,body,published_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(owner_id,source_kind,source_id) DO UPDATE SET project_id=excluded.project_id,source_url=excluded.source_url,title=excluded.title,body=excluded.body,published_at=excluded.published_at,metadata=excluded.metadata,collected_at=now()")
            .bind(user).bind(req.project_id).bind(&req.source).bind(short(&finding.id,255))
            .bind(short(&finding.url,2048)).bind(short(&finding.title,500)).bind(short(&finding.body,1200))
            .bind(finding.published).bind(meta).execute(&mut *tx).await.map_err(internal)?;
        saved+=1;
    }
    tx.commit().await.map_err(internal)?;
    Ok(Json(json!({"source":req.source,"saved":saved,"matched":items.len(),
        "note":"Read-only research evidence; verify context, licenses and original discussions before acting."})))
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn bluesky_skips_opted_out_authors_and_bad_record_keys(){
        let v=json!({"posts":[
            {"uri":"at://did:plc:abc/app.bsky.feed.post/3kabc","author":{"handle":"alice.bsky.social","labels":[]},
             "record":{"text":"invoicing is a nightmare","createdAt":"2026-10-01T00:00:00Z"},"indexedAt":"2026-10-01T00:00:01Z"},
            {"uri":"at://did:plc:def/app.bsky.feed.post/3kdef","author":{"handle":"bob.bsky.social","labels":[{"val":"!no-unauthenticated"}]},
             "record":{"text":"should not be collected"}},
            {"uri":"at://did:plc:ghi/app.bsky.feed.post/bad$key","author":{"handle":"eve.bsky.social"},"record":{"text":"bad key"}}
        ]});
        let found=bluesky_findings(&v);
        assert_eq!(found.len(),1);
        assert_eq!(found[0].url,"https://bsky.app/profile/alice.bsky.social/post/3kabc");
        assert!(found[0].body.contains("nightmare"));
        assert!(found[0].published.is_some());
    }
    #[test] fn bounds_and_untrusted_urls(){
        assert!(medium_tag("open-source").is_ok());
        assert!(medium_tag("../users").is_err());
        assert!(youtube_id("dQw4w9WgXcQ").is_ok());
        assert!(youtube_id("../etc/passwd").is_err());
        assert!(!supported("reddit"));
        assert!(discourse_host("127.0.0.1").is_err());
        assert!(discourse_host("server.internal").is_err());
        assert!(html_text("<b>problem</b> &amp; workaround").contains("problem & workaround"));
        assert!(get_date("2026-10-13T01:00:00Z").is_some());
    }
}
