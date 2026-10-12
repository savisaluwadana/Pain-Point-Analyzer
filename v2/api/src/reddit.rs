use crate::{oauth,AppState,ApiError,ApiResult,bad,internal};
use axum::{extract::{Query,State},http::{HeaderMap,StatusCode},Json};
use serde::{Deserialize,Serialize};
use serde_json::Value;
use sqlx::types::Uuid;

#[derive(Serialize,Deserialize,Clone)]
pub(crate) struct RedditPost {
 pub id:String, pub title:String,
 #[serde(default)] pub selftext:String,
 #[serde(default)] pub author:String,
 #[serde(default)] pub subreddit:String,
 #[serde(default)] pub permalink:String,
 #[serde(default)] pub score:i64,
 #[serde(default)] pub num_comments:i64,
 #[serde(default)] pub created_utc:f64,
}
#[derive(Deserialize)]
struct Child { data:RedditPost }
#[derive(Deserialize)]
struct Children { children:Vec<Child>,after:Option<String> }
#[derive(Deserialize)]
struct Listing { data:Children }
#[derive(Deserialize)]
pub struct PostQuery { subreddit:String,limit:Option<i32>,after:Option<String> }
#[derive(Deserialize)]
pub struct CommentsQuery {post_id:String,limit:Option<i32>}
fn subreddit_ok(name:&str)->bool{
 (2..=21).contains(&name.len()) && name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')
}
fn id_ok(id:&str)->bool{
 (1..=16).contains(&id.len())&&id.bytes().all(|b|b.is_ascii_alphanumeric())
}
async fn reddit_get(st:&AppState,user:Uuid,path:&str,query:Vec<(&str,String)>)->ApiResult<Value>{
 let token=oauth::access_token(st,user).await?;
 let url=format!("https://oauth.reddit.com{path}");
 let response=st.http.get(&url).bearer_auth(token).header("User-Agent",&st.cfg.user_agent).query(&query).send().await.map_err(internal)?;
 let status=response.status();
 if status==reqwest::StatusCode::TOO_MANY_REQUESTS {
   return Err(ApiError(StatusCode::TOO_MANY_REQUESTS,"Reddit API rate limit reached. Retry after the provider's allowed window.".into()));
 }
 if status==reqwest::StatusCode::UNAUTHORIZED||status==reqwest::StatusCode::FORBIDDEN {
   return Err(ApiError(StatusCode::BAD_GATEWAY,"Reddit denied access; reconnect or check app permissions".into()));
 }
 if !status.is_success(){return Err(ApiError(StatusCode::BAD_GATEWAY,format!("Reddit returned HTTP {status}")));}
 response.json().await.map_err(internal)
}
pub(crate) async fn fetch_posts(st:&AppState,user:Uuid,sub:&str,limit:i32)->ApiResult<Vec<RedditPost>>{
 if !subreddit_ok(sub){return Err(bad("Invalid subreddit"));}
 let value=reddit_get(st,user,&format!("/r/{sub}/top"),vec![("limit",limit.clamp(1,100).to_string()),("t","month".into()),("raw_json","1".into())]).await?;
 let listing:Listing=serde_json::from_value(value).map_err(internal)?;
 Ok(listing.data.children.into_iter().map(|v|v.data).collect())
}
pub async fn posts(State(st):State<AppState>,headers:HeaderMap,Query(q):Query<PostQuery>)->ApiResult<Json<Value>>{
 let user=oauth::user_id(&st,&headers).await?;
 if !subreddit_ok(&q.subreddit){return Err(bad("Invalid subreddit"));}
 if q.after.as_ref().is_some_and(|a|a.len()>64||!a.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')) {
    return Err(bad("Invalid pagination cursor"));
 }
 let mut query=vec![("limit",q.limit.unwrap_or(25).clamp(1,100).to_string()),("t","month".into()),("raw_json","1".into())];
 if let Some(after)=q.after {query.push(("after",after));}
 let result=reddit_get(&st,user,&format!("/r/{}/top",&q.subreddit),query).await?;
 let listing:Listing=serde_json::from_value(result).map_err(internal)?;
 Ok(Json(serde_json::json!({"posts":listing.data.children.into_iter().map(|c|c.data).collect::<Vec<_>>(),"after":listing.data.after})))
}
pub async fn comments(State(st):State<AppState>,headers:HeaderMap,Query(q):Query<CommentsQuery>)->ApiResult<Json<Value>>{
 let user=oauth::user_id(&st,&headers).await?;
 if !id_ok(&q.post_id){return Err(bad("Invalid post ID"));}
 let response=reddit_get(&st,user,&format!("/comments/{}",q.post_id),vec![("limit",q.limit.unwrap_or(30).clamp(1,100).to_string()),("raw_json","1".into())]).await?;
 Ok(Json(response))
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn bounded_names(){assert!(subreddit_ok("rust"));assert!(!subreddit_ok("rust/path"));assert!(!id_ok("../secret"));}
}
