mod jobs;
mod oauth;
mod reddit;
mod research;
mod scoring;

use axum::{http::StatusCode, response::{IntoResponse,Response}, Json, Router, routing::{get,post}};
use reqwest::Client;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions,PgPool};
use std::{env,net::SocketAddr,sync::Arc,time::Duration};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub(crate) struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub web_url: String,
    pub user_agent: String,
    pub encryption_key: [u8; 32],
    pub reddit_approved: bool,
    pub secure_cookies: bool,
}
#[derive(Clone)]
pub(crate) struct AppState {
    pub db: PgPool,
    pub http: Client,
    pub cfg: Arc<Config>,
}
#[derive(Debug)]
pub(crate) struct ApiError(pub StatusCode, pub String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
pub(crate) type ApiResult<T> = Result<T,ApiError>;
pub(crate) fn internal(error: impl std::fmt::Display)->ApiError {
    tracing::error!(error=%error,"api failure");
    ApiError(StatusCode::INTERNAL_SERVER_ERROR,"Internal server error".into())
}
pub(crate) fn bad(message:&str)->ApiError {ApiError(StatusCode::BAD_REQUEST,message.into())}
pub(crate) fn unauthorized()->ApiError {ApiError(StatusCode::UNAUTHORIZED,"Sign in with Reddit".into())}

fn config()->anyhow::Result<Config> {
    let key=hex::decode(env::var("TOKEN_ENCRYPTION_KEY_HEX")?)?;
    anyhow::ensure!(key.len()==32, "TOKEN_ENCRYPTION_KEY_HEX must contain exactly 32 bytes");
    let mut encryption_key=[0u8;32];
    encryption_key.copy_from_slice(&key);
    let redirect_uri=env::var("REDDIT_REDIRECT_URI")?;
    anyhow::ensure!(redirect_uri.starts_with("http://localhost:") || redirect_uri.starts_with("https://"),"Callback must be HTTPS (localhost permitted for development)");
    Ok(Config {
        client_id:env::var("REDDIT_CLIENT_ID").unwrap_or_default(),
        client_secret:env::var("REDDIT_CLIENT_SECRET").unwrap_or_default(),
        redirect_uri,
        web_url:env::var("WEB_URL").unwrap_or_else(|_|"http://localhost:3000".into()),
        user_agent:env::var("REDDIT_USER_AGENT").unwrap_or_default(),
        encryption_key,
        reddit_approved:env::var("REDDIT_API_APPROVED").as_deref()==Ok("true"),
        secure_cookies:env::var("COOKIE_SECURE").as_deref()==Ok("true"),
    })
}

async fn health(axum::extract::State(st):axum::extract::State<AppState>)->impl IntoResponse {
    match sqlx::query_scalar::<_,i32>("SELECT 1").fetch_one(&st.db).await {
        Ok(_)=>(StatusCode::OK,Json(json!({"ok":true,"database":"ready"}))),
        Err(_)=>(StatusCode::SERVICE_UNAVAILABLE,Json(json!({"ok":false,"database":"unavailable"}))),
    }
}

#[tokio::main]
async fn main()->anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_|EnvFilter::new("info"))).init();
    let cfg=config()?;
    let db=PgPoolOptions::new().max_connections(10).acquire_timeout(Duration::from_secs(10)).connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!("../db").run(&db).await?;
    let http=Client::builder().timeout(Duration::from_secs(15)).build()?;
    let st=AppState{db,http,cfg:Arc::new(cfg)};
    let app=Router::new()
      .route("/api/health",get(health))
      .route("/api/auth/reddit/start",get(oauth::start))
      .route("/api/auth/reddit/callback",get(oauth::callback))
      .route("/api/auth/logout",post(oauth::logout))
      .route("/api/me",get(oauth::me))
      .route("/api/reddit/posts",get(reddit::posts))
      .route("/api/reddit/comments",get(reddit::comments))
      .route("/api/projects",get(research::projects).post(research::create_project))
      .route("/api/evidence",get(research::evidence).post(research::ingest_evidence))
      .route("/api/research/scan",post(research::scan))
      .route("/api/research/thread",post(research::thread))
      .route("/api/research/analyze",post(research::analyze))
      .route("/api/research/runs",get(research::runs))
      .route("/api/research/jobs",get(jobs::list).post(jobs::create))
      .route("/api/opportunities",get(research::opportunities))
      .route("/api/privacy/delete-data",post(research::delete_data))
      .layer(TraceLayer::new_for_http())
      .with_state(st.clone());
    tokio::spawn(jobs::worker(st.clone()));
    tokio::spawn(jobs::cleanup(st.clone()));
    let addr=SocketAddr::from(([0,0,0,0],env::var("PORT").ok().and_then(|s|s.parse().ok()).unwrap_or(4000)));
    tracing::info!(%addr,"Rust API listening");
    let listener=tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener,app).with_graceful_shutdown(async {let _=tokio::signal::ctrl_c().await;}).await?;
    Ok(())
}
