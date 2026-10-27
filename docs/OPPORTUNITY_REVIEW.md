# Opportunity review workflow (first implementation)

The **Open-source project ideas** screen now lets an authenticated local operator inspect a saved candidate,
review its supporting evidence, record existing solutions and their own validation notes, and set a **manual**
decision. This is a contribution-planning tool, not an automated demand-verification or commercial prospecting feature.

## Review stages

- `unreviewed` — default, no review completed
- `investigating` — reading original context and finding alternatives
- `needs_validation` — further independent research or interviews needed
- `ready_to_build` — explicit human decision, requiring at least 20 characters of justification
- `not_pursuing` — decided not to build

The decision is separate from the within-run analysis score. Even `ready_to_build` is **not** a claim of
statistically verified demand or willingness to pay.

## Routes and storage

- `GET /api/opportunities` includes the decision, user-authored notes, and last review time.
- `GET /api/opportunities/{id}/brief` resolves the owning research run and uses the existing
  hash-verified report hydration. It returns up to five representative excerpts when the original
  evidence still exists, is unchanged, and belongs to the operator. It does not persist copied text.
- `PUT /api/opportunities/{id}/review` accepts `decision`, `existing_solutions` (max 2,000
  characters), and `validation_notes` (max 4,000 characters). Only the authenticated owner can edit it.

Migration `009_opportunity_review.sql` adds the fields with database validation constraints.
User-authored notes are included in the existing personal data export; account deletion cascades them.
Avoid pasting private details, third-party post bodies, or copyrighted excerpts into notes.

## Known limitations and future improvements

This first step does **not** automatically identify competitors, interview users, or infer buyer intent.
It shows representative report evidence, not every collected post. Opportunity titles still come from
the current keyword categories, and older reports may have no hash-verified representative evidence.
The semantic model, human-labeled evaluation corpus, competitor discovery and compliant saved watches
remain separate future work. Do not deploy this local-token alpha on a public origin.

## Verification

Run the standard CI commands from `docs/ANALYSIS_QUALITY.md`. The frontend component tests cover
the review panel and PUT payload; Rust unit tests validate decision and size constraints. A production
security and provider-content permissions review is still required.
