//! Browser session cookie CSRF guard. MCP bearer-token calls remain cookie independent.
use crate::AppState;
use axum::{
 extract::{Request,State},
 http::{header::{COOKIE,ORIGIN},HeaderMap,Method,StatusCode},
 middleware::Next,
 response::Response,
};
fn is_session_cookie(headers:&HeaderMap)->bool{
 headers.get_all(COOKIE).iter().filter_map(|v|v.to_str().ok())
  .flat_map(|v|v.split(';')).any(|v|{
    v.trim().split_once('=').is_some_and(|(name,value)|name=="pil_session"&&!value.is_empty())
  })
}
pub(crate) fn allowed(method:&Method,headers:&HeaderMap,web_url:&str)->bool{
 if matches!(*method,Method::GET|Method::HEAD|Method::OPTIONS){return true;}
 if !is_session_cookie(headers){return true;}
 headers.get(ORIGIN).and_then(|v|v.to_str().ok())
  .is_some_and(|origin|origin==web_url.trim_end_matches('/'))
}
pub(crate) async fn require_same_origin(
 State(state):State<AppState>,req:Request,next:Next
)->Result<Response,StatusCode>{
 if !allowed(req.method(),req.headers(),&state.cfg.web_url){return Err(StatusCode::FORBIDDEN);}
 Ok(next.run(req).await)
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn blocks_cross_site_and_missing_origin_for_cookie_mutations(){
  let mut headers=HeaderMap::new();
  headers.insert(COOKIE,"pil_session=test".parse().unwrap());
  assert!(!allowed(&Method::POST,&headers,"http://localhost:3000"));
  headers.insert(ORIGIN,"https://evil.example".parse().unwrap());
  assert!(!allowed(&Method::DELETE,&headers,"http://localhost:3000"));
  assert!(allowed(&Method::GET,&headers,"http://localhost:3000"));
  headers.insert(ORIGIN,"http://localhost:3000".parse().unwrap());
  assert!(allowed(&Method::POST,&headers,"http://localhost:3000"));
  assert!(allowed(&Method::DELETE,&headers,"http://localhost:3000"));
 }
 #[test]
 fn bearer_tool_calls_without_cookies_are_not_blocked(){
  let headers=HeaderMap::new();
  assert!(allowed(&Method::POST,&headers,"http://localhost:3000"));
  let mut headers=HeaderMap::new();
  headers.insert(COOKIE,"pil_oauth_state=foo".parse().unwrap());
  assert!(allowed(&Method::POST,&headers,"http://localhost:3000"));
 }
}