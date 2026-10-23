use crate::{bad, internal, oauth, ApiError, ApiResult, AppState};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RedditPost {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub selftext: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub subreddit: String,
    #[serde(default)]
    pub permalink: String,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub num_comments: i64,
    #[serde(default)]
    pub created_utc: f64,
}
#[derive(Deserialize)]
struct Child {
    data: RedditPost,
}
#[derive(Deserialize)]
struct Children {
    children: Vec<Child>,
    after: Option<String>,
}
#[derive(Deserialize)]
struct Listing {
    data: Children,
}
#[derive(Deserialize)]
pub struct PostQuery {
    subreddit: String,
    limit: Option<i32>,
    after: Option<String>,
}
#[derive(Deserialize)]
pub struct CommentsQuery {
    post_id: String,
    limit: Option<i32>,
}
fn subreddit_ok(name: &str) -> bool {
    (2..=21).contains(&name.len()) && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn id_ok(id: &str) -> bool {
    (1..=16).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric())
}
pub(crate) fn is_reddit_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host == "reddit.com" || host.ends_with(".reddit.com") || host == "redd.it" || host.ends_with(".redd.it")
}
fn configured_limit(name: &str, default: i32, max: i32) -> i32 {
    std::env::var(name)
        .ok()
        .and_then(|x| x.parse::<i32>().ok())
        .unwrap_or(default)
        .clamp(1, max)
}
async fn reserve_provider_request(st: &AppState, user: Uuid) -> ApiResult<()> {
    let mut tx = st.db.begin().await.map_err(internal)?;
    let scopes = [
        (
            "global".to_string(),
            None,
            configured_limit("REDDIT_REQUESTS_PER_MINUTE", 30, 120),
        ),
        (
            format!("user:{user}"),
            Some(user),
            configured_limit("REDDIT_USER_REQUESTS_PER_MINUTE", 10, 60),
        ),
    ];
    for (scope, owner, limit) in scopes {
        sqlx::query("INSERT INTO reddit_api_budgets(scope,owner_id) VALUES($1,$2) ON CONFLICT DO NOTHING")
            .bind(&scope)
            .bind(owner)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        let (started, count, blocked): (DateTime<Utc>, i32, Option<DateTime<Utc>>) = sqlx::query_as(
            "SELECT window_started_at,request_count,blocked_until FROM reddit_api_budgets WHERE scope=$1 FOR UPDATE",
        )
        .bind(&scope)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
        if blocked.is_some_and(|until| until > Utc::now()) {
            return Err(ApiError(
                StatusCode::TOO_MANY_REQUESTS,
                "Reddit provider backoff is active. Retry later.".into(),
            ));
        }
        let elapsed = Utc::now().signed_duration_since(started).num_seconds() >= 60;
        if !elapsed && count >= limit {
            return Err(ApiError(
                StatusCode::TOO_MANY_REQUESTS,
                "This account or app has reached its conservative Reddit API request budget. Retry in about a minute."
                    .into(),
            ));
        }
        sqlx::query(
            "UPDATE reddit_api_budgets SET request_count=$2,window_started_at=CASE WHEN $3 THEN now() ELSE \
                window_started_at END,blocked_until=CASE WHEN blocked_until<now() THEN NULL ELSE blocked_until END \
                WHERE scope=$1",
        )
        .bind(&scope)
        .bind(if elapsed { 1 } else { count + 1 })
        .bind(elapsed)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    }
    tx.commit().await.map_err(internal)?;
    Ok(())
}
async fn backoff_after_provider_response(st: &AppState, response: &reqwest::Response) -> ApiResult<()> {
    // Apply provider backoff globally: multiple workers share the same app credentials.
    let limited = response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        || response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok())
            .is_some_and(|remaining| remaining <= 0.0);
    if limited {
        let delay = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| {
                response
                    .headers()
                    .get("x-ratelimit-reset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|v| v.ceil() as i64)
            })
            .unwrap_or(60)
            .clamp(5, 3600);
        sqlx::query(
            "UPDATE reddit_api_budgets SET blocked_until=GREATEST(COALESCE(blocked_until,now()),now()+($1 * \
                interval '1 second')) WHERE scope='global'",
        )
        .bind(delay)
        .execute(&st.db)
        .await
        .map_err(internal)?;
    }
    Ok(())
}

// Permanent access denials are not transient upstream outages. Preserve a distinct
// status so the background worker can stop instead of repeatedly retrying a 401/403.
fn reddit_response_error(status: reqwest::StatusCode) -> Option<ApiError> {
    match status {
        reqwest::StatusCode::TOO_MANY_REQUESTS => Some(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Reddit API rate limit reached. Retry after the provider's allowed window.".into(),
        )),
        reqwest::StatusCode::UNAUTHORIZED => Some(ApiError(
            StatusCode::FAILED_DEPENDENCY,
            "Reddit rejected the access token; reconnect the account before collecting again.".into(),
        )),
        reqwest::StatusCode::FORBIDDEN => Some(ApiError(
            StatusCode::FAILED_DEPENDENCY,
            "Reddit denied API access; verify the application's approval and permissions.".into(),
        )),
        _ if !status.is_success() => Some(ApiError(
            StatusCode::BAD_GATEWAY,
            format!("Reddit returned HTTP {status}"),
        )),
        _ => None,
    }
}

async fn reddit_get(st: &AppState, user: Uuid, path: &str, query: Vec<(&str, String)>) -> ApiResult<Value> {
    let token = oauth::access_token(st, user).await?;
    reserve_provider_request(st, user).await?;
    let url = format!("https://oauth.reddit.com{path}");
    let response = st
        .http
        .get(&url)
        .bearer_auth(token)
        .header("User-Agent", &st.cfg.user_agent)
        .query(&query)
        .send()
        .await
        .map_err(internal)?;
    let status = response.status();
    backoff_after_provider_response(st, &response).await?;
    if let Some(error) = reddit_response_error(status) {
        return Err(error);
    }
    response.json().await.map_err(internal)
}
pub(crate) async fn fetch_posts(st: &AppState, user: Uuid, sub: &str, limit: i32) -> ApiResult<Vec<RedditPost>> {
    if !subreddit_ok(sub) {
        return Err(bad("Invalid subreddit"));
    }
    let value = reddit_get(
        st,
        user,
        &format!("/r/{sub}/top"),
        vec![
            ("limit", limit.clamp(1, 100).to_string()),
            ("t", "month".into()),
            ("raw_json", "1".into()),
        ],
    )
    .await?;
    let listing: Listing = serde_json::from_value(value).map_err(internal)?;
    Ok(listing.data.children.into_iter().map(|v| v.data).collect())
}
pub async fn posts(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<PostQuery>,
) -> ApiResult<Json<Value>> {
    let user = oauth::user_id(&st, &headers).await?;
    if !subreddit_ok(&q.subreddit) {
        return Err(bad("Invalid subreddit"));
    }
    if q.after
        .as_ref()
        .is_some_and(|a| a.len() > 64 || !a.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
    {
        return Err(bad("Invalid pagination cursor"));
    }
    let mut query = vec![
        ("limit", q.limit.unwrap_or(25).clamp(1, 100).to_string()),
        ("t", "month".into()),
        ("raw_json", "1".into()),
    ];
    if let Some(after) = q.after {
        query.push(("after", after));
    }
    let result = reddit_get(&st, user, &format!("/r/{}/top", q.subreddit), query).await?;
    let listing: Listing = serde_json::from_value(result).map_err(internal)?;
    Ok(Json(serde_json::json!({
        "posts": listing.data.children.into_iter().map(|c| c.data).collect::<Vec<_>>(),
        "after": listing.data.after,
    })))
}
pub async fn comments(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<CommentsQuery>,
) -> ApiResult<Json<Value>> {
    let user = oauth::user_id(&st, &headers).await?;
    if !id_ok(&q.post_id) {
        return Err(bad("Invalid post ID"));
    }
    let response = reddit_get(
        &st,
        user,
        &format!("/comments/{}", q.post_id),
        vec![
            ("limit", q.limit.unwrap_or(30).clamp(1, 100).to_string()),
            ("raw_json", "1".into()),
        ],
    )
    .await?;
    Ok(Json(response))
}
#[cfg(test)]
mod tests {
    // Unwraps in these fixture tests are assertions: a failure must fail the test.
    use super::*;
    #[test]
    fn bounded_names() {
        assert!(subreddit_ok("rust"));
        assert!(!subreddit_ok("rust/path"));
        assert!(!id_ok("../secret"));
        assert!(is_reddit_host("www.reddit.com"));
        assert!(!is_reddit_host("fake-reddit.com"));
        assert_eq!(configured_limit("UNSET_TEST_SETTING", 30, 120), 30);
    }
    #[test]
    fn provider_errors_distinguish_permanent_denials_and_rate_limits() {
        assert_eq!(
            reddit_response_error(reqwest::StatusCode::UNAUTHORIZED).unwrap().0,
            StatusCode::FAILED_DEPENDENCY
        );
        assert_eq!(
            reddit_response_error(reqwest::StatusCode::FORBIDDEN).unwrap().0,
            StatusCode::FAILED_DEPENDENCY
        );
        assert_eq!(
            reddit_response_error(reqwest::StatusCode::TOO_MANY_REQUESTS).unwrap().0,
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            reddit_response_error(reqwest::StatusCode::SERVICE_UNAVAILABLE)
                .unwrap()
                .0,
            StatusCode::BAD_GATEWAY
        );
        assert!(reddit_response_error(reqwest::StatusCode::OK).is_none());
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RedditComment {
    pub id: String,
    pub body: String,
    pub author: String,
    pub permalink: String,
    pub score: i64,
    pub created_utc: f64,
}
fn flatten_comments(node: &Value, out: &mut Vec<RedditComment>, limit: usize, depth: u8) {
    if depth >= 8 || out.len() >= limit {
        return;
    }
    if let Some(children) = node.pointer("/data/children").and_then(Value::as_array) {
        for child in children {
            if out.len() >= limit {
                break;
            }
            if child.get("kind").and_then(Value::as_str) != Some("t1") {
                continue;
            }
            let data = &child["data"];
            let id = data["id"].as_str().unwrap_or_default();
            let body = data["body"].as_str().unwrap_or_default();
            if id_ok(id) && !body.is_empty() && body != "[deleted]" && body != "[removed]" {
                out.push(RedditComment {
                    id: id.to_owned(),
                    body: body.chars().take(10000).collect(),
                    author: data["author"].as_str().unwrap_or_default().to_owned(),
                    permalink: data["permalink"].as_str().unwrap_or_default().to_owned(),
                    score: data["score"].as_i64().unwrap_or(0),
                    created_utc: data["created_utc"].as_f64().unwrap_or(0.0),
                });
            }
            if data["replies"].is_object() {
                flatten_comments(&data["replies"], out, limit, depth + 1);
            }
        }
    }
}
pub(crate) async fn fetch_comments(
    st: &AppState,
    user: Uuid,
    post_id: &str,
    limit: usize,
) -> ApiResult<Vec<RedditComment>> {
    if !id_ok(post_id) {
        return Err(bad("Invalid post ID"));
    }
    let payload = reddit_get(
        st,
        user,
        &format!("/comments/{post_id}"),
        vec![("limit", limit.min(100).to_string()), ("raw_json", "1".into())],
    )
    .await?;
    let mut comments = Vec::new();
    if let Some(list) = payload.get(1) {
        flatten_comments(list, &mut comments, limit.min(100), 0);
    }
    Ok(comments)
}
#[cfg(test)]
mod comment_tests {
    use super::*;
    #[test]
    fn nested_comments_are_bounded_and_exclude_deleted() {
        let value = serde_json::json!({
            "data": {
                "children": [
                    {
                        "kind": "t1",
                        "data": {
                            "id": "abc",
                            "body": "manual workaround",
                            "replies": {
                                "data": {
                                    "children": [{
                                        "kind": "t1",
                                        "data": {
                                            "id": "def",
                                            "body": "[deleted]",
                                        },
                                    }],
                                },
                            },
                        },
                    },
                    {
                        "kind": "more",
                        "data": {
                            "id": "ghi",
                        },
                    },
                ],
            },
        });

        let mut out = Vec::new();
        flatten_comments(&value, &mut out, 25, 0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "abc");
    }
}
