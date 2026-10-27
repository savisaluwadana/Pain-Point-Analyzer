//! Owner-scoped opportunity review. Decisions are user-entered, not AI-validated demand.
use crate::{bad, internal, oauth, research, ApiError, ApiResult, AppState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;

#[derive(Deserialize)]
pub struct ReviewInput {
    decision: String,
    validation_notes: String,
    existing_solutions: String,
}

fn validate_review(body: &ReviewInput) -> ApiResult<()> {
    if !matches!(
        body.decision.as_str(),
        "unreviewed" | "investigating" | "needs_validation" | "ready_to_build" | "not_pursuing"
    ) {
        return Err(bad("Invalid opportunity decision"));
    }
    if body.validation_notes.chars().count() > 4000 || body.existing_solutions.chars().count() > 2000 {
        return Err(bad(
            "Validation notes must be at most 4000 characters and existing solutions at most 2000",
        ));
    }
    if body.decision == "ready_to_build" && body.validation_notes.trim().chars().count() < 20 {
        return Err(bad(
            "Record the basis for the ready-to-build decision (at least 20 characters)",
        ));
    }
    Ok(())
}

pub async fn save_review(
    State(st): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<ReviewInput>,
) -> ApiResult<Json<Value>> {
    let uid = oauth::user_id(&st, &headers).await?;
    validate_review(&body)?;
    let updated: Option<Uuid> = sqlx::query_scalar(
        "UPDATE opportunities SET decision=$3, validation_notes=$4, existing_solutions=$5, reviewed_at=now() \
         WHERE owner_id=$1 AND id=$2 RETURNING id",
    )
    .bind(uid)
    .bind(id)
    .bind(&body.decision)
    .bind(body.validation_notes.trim())
    .bind(body.existing_solutions.trim())
    .fetch_optional(&st.db)
    .await
    .map_err(internal)?;
    if updated.is_none() {
        return Err(ApiError(StatusCode::NOT_FOUND, "Opportunity not found".into()));
    }
    Ok(Json(json!({"saved": true, "id": id})))
}

/// Return only an existing research run's representative, still-valid references.
/// Do not persist copied quotes in opportunities or user-authored review notes.
pub async fn brief(State(st): State<AppState>, headers: HeaderMap, Path(id): Path<Uuid>) -> ApiResult<Json<Value>> {
    let uid = oauth::user_id(&st, &headers).await?;
    let row: Option<(String, Value, Value)> = sqlx::query_as(
        "SELECT o.title,o.evidence_refs,r.report FROM opportunities o \
         JOIN research_runs r ON r.id=o.run_id AND r.owner_id=o.owner_id \
         WHERE o.owner_id=$1 AND o.id=$2",
    )
    .bind(uid)
    .bind(id)
    .fetch_optional(&st.db)
    .await
    .map_err(internal)?;
    let (title, refs, mut report) =
        row.ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "Opportunity report not found".into()))?;
    research::hydrate_report(&st, uid, &mut report).await?;
    let cluster = report["clusters"]
        .as_array()
        .and_then(|clusters| clusters.iter().find(|c| c["category"].as_str() == Some(title.as_str())))
        .cloned();
    Ok(Json(json!({
        "title": title,
        "cluster": cluster,
        "engine": report["engine"],
        "coverage": report["coverage"],
        "referenced_count": refs.as_array().map_or(0, Vec::len),
        "caveat": "Historical counts are hypotheses, not evidence of willingness to pay. Only unchanged, owner-scoped representative excerpts are shown. Deleted, expired or edited evidence is unavailable."
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(decision: &str, notes: &str, solutions: &str) -> ReviewInput {
        ReviewInput {
            decision: decision.into(),
            validation_notes: notes.into(),
            existing_solutions: solutions.into(),
        }
    }

    #[test]
    fn review_decisions_are_explicit_and_bounded() {
        assert!(validate_review(&input("investigating", "", "")).is_ok());
        assert!(validate_review(&input("needs_validation", "", "")).is_ok());
        assert!(validate_review(&input("ready_to_build", "Interviewed users and recorded results", "")).is_ok());
        assert!(validate_review(&input("ready_to_build", "Looks popular", "")).is_err());
        assert!(validate_review(&input("verified_demand", "", "")).is_err());
        assert!(validate_review(&input("investigating", &"x".repeat(4001), "")).is_err());
        assert!(validate_review(&input("investigating", "", &"x".repeat(2001))).is_err());
    }
}
