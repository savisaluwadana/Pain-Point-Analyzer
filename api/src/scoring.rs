//! API-specific concurrency boundary around the shared, deterministic analyzer.
#[path = "../../shared/scoring.rs"]
mod core;
pub use core::*;

static ANALYSIS_SLOT: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
/// CPU-heavy analysis must not block Axum workers or run unbounded concurrent copies.
pub async fn analyze_async(evidence: Vec<EvidenceText>) -> crate::ApiResult<Report> {
    let permit = ANALYSIS_SLOT.try_acquire().map_err(|_| {
        crate::ApiError(
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Another analysis is running; retry after it completes".into(),
        )
    })?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        analyze(&evidence)
    })
    .await
    .map_err(crate::internal)
}
