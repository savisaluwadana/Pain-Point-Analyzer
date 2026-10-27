# Analysis design proposal

Status: **Phase 3 foundation implemented; semantic inference remains proposed.** The keyword engine now has
word/phrase boundaries, limited clause negation, stable transitive copy/thread grouping, versioned logarithmic
ranking, explicit coverage and up to 10,000-record opt-in budgets. Supporting excerpts are hydrated from currently
owned evidence rather than stored in reports. HTTP and MCP analysis share a single CPU-work permit.

The embedding model, cache, semantic clustering, recency-weighted ranking, full held-out semantic evaluation and
cross-architecture packaging described below are **not implemented**. The current `keywords-v2` formula is documented
in [analysis quality](ANALYSIS_QUALITY.md). The original review below describes the pre-fix baseline, not current code.

## Recommendation

Use local, quantized **all-MiniLM-L6-v2 embeddings with average-linkage agglomerative clustering** as the default. Label
clusters with deterministic TF-IDF terms. Use corrected keyword rules as the dependency-free fallback. Keep LLM labels
and summaries as an explicitly requested, optional overlay through the existing MCP host path. No paid key, GPU, Python
service or inference network connection is required for the default.

This is a candidate architecture, not a claim of measured improvement. The evaluation and packaging gates below must
pass before making it the default. If they fail, retain the corrected keyword engine and report the failure.

## What the current code does

Reviewed against main after Phase 2:

- [scoring.rs](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/api/src/scoring.rs) uses substring matches against eight fixed categories. `terror` can match `error`;
  neutral mentions of `pricing` can count as pain. Topic relevance also uses lexical substring checks.
- Its score is `min(100, 15 * independent_stories + 3 * hits)`. Six one-hit stories already exceed 100 before capping.
  Ties do not have a stable secondary ordering. Cluster `examples` are empty.
- Same-thread grouping and long normalized copied-text grouping exist. Copy grouping depends on input order; both must
  survive the replacement with stable, transitive grouping.
- [HTTP analysis](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/api/src/research.rs) selects at most 1,000 recent items. [MCP analysis](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/api/src/mcp/evidence.rs)
  clamps its item limit to 500. Replacing the scorer alone would leave these dataset limits in place.
- The scoring input lacks publication dates. Existing saved reports and opportunities contain scores and evidence
  references. The frontend presents scores out of 100 without explaining their components.
- [MCP synthesis](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/api/src/mcp/advanced.rs) already supports host-provided annotations and synthesis tied to owned
  evidence IDs. It does not run an LLM inside the API.

The proposal changes analysis only. Collection permissions, provider budgets, retention, tenant boundaries, auth, CSRF,
localhost bindings, source-specific exports and the data-use policy remain requirements.

## Compare the three approaches

### 1. Local embeddings, clustering and TF-IDF labels

Embeddings group paraphrases without requiring shared keywords. They can also group neutral product discussion with
complaints, confuse opposite sentiments and lose small themes. Similarity is not proof of a pain point; complaint
signals, noise handling, evidence and evaluation are still needed.

[MiniLM's model card][minilm] describes a 384-dimensional English sentence/short-paragraph encoder, with default
truncation beyond 256 word pieces. Its license is Apache-2.0. A pinned [quantized ONNX export][weights] is about 23 MB
for weights alone. [bge-small-en-v1.5][bge] is another small English, 384-dimensional option, licensed MIT; its longer
context is useful, but superiority on this dataset has not been established. Benchmark it only if MiniLM fails the
evaluation gate; do not silently switch models between runs.

Use [fastembed-rs][fastembed] for tokenization and ONNX inference, with local model files through its [user-defined
model API][local-model]. Pin the crate, ONNX runtime, weights, tokenizer, pooling and normalization. The library's
default features include model-hub/runtime download support; explicitly configure the selected release so analysis
cannot fetch missing artifacts at runtime.

Compare clustering choices:

- **Average linkage, recommended:** start from cosine distances between normalized story vectors. Merge by average
  inter-cluster distance and cut at a fixed, evaluated distance threshold. No fixed cluster count or random
  initialization is needed. [Kodama][kodama] provides a Rust implementation. Its quadratic distance storage is
  predictable at 10,000 stories but requires a memory/concurrency budget. Test equal-distance ties.
- **HDBSCAN:** can identify noise and varying-density groups without selecting a cluster count. It introduces density
  parameters and additional implementation/performance validation for high-dimensional vectors. Compare it if average
  linkage repeatedly merges unrelated themes or discards sparse themes. Do not add a Python service just for clustering.
  See the [HDBSCAN reference][hdbscan] for algorithm parameters and behavior.

Use average linkage with cosine distance, not Ward linkage with cosine. Reject groups with fewer than three independent
stories as insufficient evidence. Keep those observations visible as unclustered items. A distance cutoff is not a
probability; select it on the development split, record it and freeze it before held-out evaluation.

Labels use the highest-scoring one- and two-word TF-IDF terms from independent stories, with a versioned stopword list
and lexical tie-breaking. Calculate IDF over the selected dataset, not only the cluster. Display the terms as a
tentative label, with representative quotes for interpretation. No generated assertion of pain is justified solely by a
topic cluster.

### 2. Optional LLM labeling and summarization

Use the existing MCP host annotation/synthesis tools after deterministic clustering. Supply bounded evidence and IDs
through that authorized path; validate returned IDs against the run's owner and selected evidence. Labels and summaries
can explain a cluster better than TF-IDF, but can hallucinate, follow instructions inside source text or overstate the
evidence. Treat source text as data, validate structure and preserve provenance; these measures do not prove a summary
correct.

The host decides its model and whether processing is local or external. Remote use can require a paid key and disclose
supplied evidence, so it must be explicitly selected and subject to existing source-use permissions. Do not
automatically send evidence because a key is configured. An optional direct API adapter could be added later, but is not
necessary for this implementation and is not the default.

LLM overlays must not change deterministic membership, ranking or base labels. Store the chosen overlay and model
metadata separately; do not promise reproducible regeneration, even at temperature zero. Skip unavailable or rejected
overlays without blocking local analysis. Retention rules also apply to summaries containing source text.

### 3. Corrected keywords, fallback

Keep eight familiar categories, but use normalized tokens and phrases rather than substring matching. Normalize
apostrophes and case; match word boundaries, so `terror` does not match `error`. Remove standalone `can't` and
`pricing`. Require specific complaint context for otherwise neutral category terms.

Apply clause-local negation to matched phrases: `not slow` and `no issues` must not count as performance or reliability
complaints. Handle `not only slow` and `not slow, but expensive` separately. Record matched phrases and negation
decisions so users can inspect the result. Include these cases in tests; a short negation window alone is insufficient.
Sarcasm and complex syntax remain limitations.

This engine needs no model/runtime dependency or download. It is cheap, deterministic and easy to audit, but misses new
themes and paraphrases. Permit overlapping categories and an unclassified result. Share evidence grouping, ranking,
quote selection and coverage reporting with the semantic engine so comparisons remain fair.

| Property     | Local embeddings          | LLM overlay        | Keywords    |
| ------------ | ------------------------- | ------------------ | ----------- |
| New themes   | Learns similarity         | Describes groups   | Fixed rules |
| Explanation  | Terms, quotes, scores     | May invent claims  | Rule trace  |
| Offline      | Bundled artifacts         | Host-dependent     | Yes         |
| Image cost   | Weights and runtime       | None for host path | Negligible  |
| 1k–10k CPU   | Inference, pair distances | Host-dependent     | Token scan  |
| External key | None                      | Optional           | None        |

## Docker, CPU and storage budget

Bundle pinned model/tokenizer artifacts, notices, digests and the matching CPU ONNX library into the API image. The
current runtime image copies only the API binary and certificates; adding a Rust dependency alone will not package its
shared libraries. Validate the Debian runtime and non-root user on supported architectures. Keep `docker compose up` as
the normal entry point and keep standard `postgres:17-alpine`.

Source builds may download dependencies and the pinned model, as they already download Cargo/npm dependencies. Once
images exist, inference and the demo must work with external network access disabled. No first-request model downloads.
Offer a documented keyword-only build for environments unable to obtain model artifacts. In explicit semantic mode,
missing/corrupt artifacts produce an actionable error. An explicit `auto` mode may select keywords, but must return and
save the actual engine and fallback reason. Never mask provider collection errors or blocked access by switching
analysis engines.

The following are engineering budgets, **not benchmark results**:

- Target machine: four CPU cores, 8 GiB RAM, no GPU. One heavy analysis job at a time; leave capacity for Postgres and
  the web server. Run CPU work outside Axum's asynchronous request executor and preserve job ownership.
- Aim for uncached, single-window input processing within 30 seconds for 1,000 observations and five minutes for 10,000.
  Measure extraction, inference and clustering separately, plus cached reruns. Long inputs increase window count and
  cost. If these targets fail, expose the measured limits before changing the default.
- For `n` independent stories, a condensed `f64` distance matrix has `n * (n - 1) / 2` entries: about 3.8 MiB at 1,000
  and 381.4 MiB at 10,000. Distance calculation also costs work proportional to vector dimension. Budget under 2 GiB
  total analysis RSS at 10,000, including model/runtime, linkage workspace and input buffers.
- A 384-element `f32` vector occupies 1,536 bytes: about 14.6 MiB for 10,000 observations before database overhead.
  Cache one combined vector per observation, not an unbounded set of windows.
- Reserve roughly 50–150 MiB additional uncompressed API image space for the model/tokenizer/runtime as an initial
  estimate. Measure actual image layers and compressed download size; the 23 MB weight file is not the total image
  increment. Check both packaging and inference on each supported CPU architecture.

## Proposed deterministic pipeline

1. **Select an authorized snapshot.** Apply owner, project, batch and explicit source/search filters before analysis.
   Select by stable date/UUID order and record scope, selected IDs/content hashes, omitted count and whether the result
   was truncated. Preserve existing routes, MCP tool names and current limit defaults. Add an optional analysis-only
   budget up to 10,000 observations with bounded internal pagination; do not increase collection budgets or turn an
   existing 500-item request into an unbounded query. Use the existing job path for expensive work. Snapshot consistency
   must cover pagination and concurrent evidence deletion.
2. **Separate selection from relevance.** Preserve explicit literal search filters. For semantic analysis, do not
   prefilter a selected project/batch using the old keyword topic gate, which would discard paraphrases before
   embedding. Record whether scope is a selected collection or an explicitly requested topic filter. Evaluate any
   semantic topic threshold separately; do not imply a bounded snapshot covers the whole corpus.
3. **Group independent stories.** Canonicalize provider/thread identities, including Reddit root/comment/archive forms.
   Union same-thread items and long normalized exact copies transitively, independently of arrival order. Preserve all
   evidence IDs. Keep the existing conservative minimum length for cross-thread copied-text matching; short common
   phrases must not collapse unrelated stories. Do not infer identity from authors.
4. **Embed content locally.** Hash the actual normalized title/body plus the model/preprocessing fingerprint. Split long
   content into deterministic windows of at most 256 tokens including special tokens, with a fixed overlap and maximum
   of four windows. Record clipping. Mean-pool as required by the model, combine windows using fixed weights, then
   normalize. Pin those details before evaluation. Reject non-finite/zero vectors. For each independent story choose its
   most central observation vector among unique content hashes, with UUID tie-breaking; repeats must not give a story
   extra weight. This can lose secondary themes inside one long discussion; report that limit.
5. **Cluster and label.** Calculate cosine distances in stable story order with fixed accumulation, round to a
   documented precision and use pinned average linkage. Test the library's handling of equal distances. Cut using the
   saved threshold, retain groups of at least three stories and expose unclustered coverage. Generate stable cluster IDs
   from sorted story keys and the engine/configuration fingerprint. Generate TF-IDF labels. Mark a story as supporting
   only when the versioned complaint rules yield positive strength. A topic group needs at least three supporting
   stories to appear as a pain cluster; otherwise expose it as a topic with insufficient complaint evidence. Preserve
   neutral members as context without adding them to support counts. This gate may miss implicit complaints; measure
   that loss in evaluation.
6. **Rank and present evidence.** Apply the shared ranking below. Select three to five distinct stories nearest the
   cluster center; break equal centrality by shorter text, then UUID. Return short, exact source excerpts (at most 240
   characters each), evidence IDs and links. Every member remains accessible through its IDs.

Save an analysis manifest: engine/version, model and tokenizer digests, preprocessing/rule versions, clustering
settings, frozen `as_of`, input digest, scope/coverage, membership and ranking components. No random sampling, UMAP or
stochastic clustering is required. Repeated runs with the same vectors, inputs, settings and runtime must match,
including ties and output ordering. Test shuffled inputs as well as repeated runs. Fresh inference on different
CPU/runtime versions is not promised to be bit-identical; preserve vector provenance and test tolerance boundaries
rather than claiming portable bitwise reproducibility.

## Ranking that does not saturate

Rank independent stories rather than comments, keyword hits or authors. Proposed version-one raw priority:

```text
priority = 0.55 * ln(1 + T) + 0.15 * ln(1 + D) + 0.15 * R + 0.15 * C
```

- `T`: independent stories supporting the theme. Each thread/copied-text group counts once.
- `D`: distinct provider families with independent supporting stories. Reddit OAuth and Reddit archive both count as
  Reddit. Copied cross-posts count once; retain their provenance without treating copies as independent corroboration.
  Assign each copied-story group one canonical provider by earliest known publication, then UUID. This is diversity of
  observations, not a credibility rating.
- `R`: mean per-story recency, `2 ^ (-age_days / 30)`, using publication dates relative to saved `as_of`.
  Unknown/invalid/future dates contribute a documented neutral 0.5 and an unknown-date count. Do not substitute a recent
  collection time for an old publication date. Use the earliest valid publication timestamp per story.
- `C`: mean per-story complaint strength in `[0, 1]`, from versioned explicit complaint/context/negation rules. Save
  contributing phrases and counts; no signal contributes zero. This is a limited rule-based proxy, not measured
  sentiment or willingness to pay. Show neutral/uncertain evidence separately from supporting stories.

These are initial weights to freeze before held-out evaluation. Thread support deliberately dominates; log growth
reduces the effect of very large themes without a hard cap. Show the raw score, each weighted component, counts, missing
dates, coverage and the formula version in the UI.

For a familiar display, calculate relative priority as the midrank percentile of raw priorities within the run. Equal
raw priorities receive equal percentiles; stable cluster IDs break presentation ties only. A run with one cluster has no
comparative ranking: show that explicitly (a neutral numeric value of 50 can serve legacy clients). Keep
`severity_score` and opportunity `score` as documented compatibility aliases, with an additive score-version field; new
UI labels say **relative priority**, not severity. Historical reports without a version retain their legacy
interpretation. Percentiles from different datasets are not comparable, and ranking is not market size.

## Cache, quotes and retention

Use an append-only migration after the existing migrations for an `analysis_embeddings` table. Do not renumber old
migrations or require pgvector: exact bounded clustering reads vectors into memory, so `bytea` is enough. Store
owner/evidence ID, content hash, model/preprocessing fingerprint, dimension, encoding and vector bytes. Use a composite
owner/evidence foreign key with cascade deletion, adding the necessary unique key to evidence in the new migration.
Validate dimensions/byte lengths and bound batch reads. No shared cache across owners.

Reuse only matching hashes and fingerprints. Evidence edits invalidate cached vectors. Retention, disconnect and
account/manual deletion must cascade to the cache; concurrent inference cannot recreate deleted evidence or insert an
orphan vector. Treat embeddings as sensitive derived data, exclude them from normal exports/logs and never use them to
train a model. Keeping vectors must not lengthen evidence retention.

**Do not persist raw quote text in historical report JSON.** Reports are saved and exported today; copying Reddit
excerpts there would outlive evidence deletion and evade source-specific export exclusions. Persist evidence IDs,
content hashes and excerpt offsets only, then hydrate authorized, still-existing evidence when reading a report. If text
changes or expires, display an unavailable-evidence marker and a quote-availability count; never fabricate a
replacement. Apply the same rule to optional LLM outputs that reproduce source text. New runs should have three to five
quotes per qualifying cluster; old/deleted evidence cannot guarantee that.

## Evaluation and release gates (Step C)

Create approximately 100 original fictional observations in `examples/eval/`, with an explicit license and provenance
statement: 80 observations across ten themes, plus 20 neutral/noise observations. Include paraphrases, lexical traps,
negations, mixed sentiment, long inputs, same-thread replies and cross-source copied text. Annotate evidence and
independent-story identities, theme labels, complaint status and expected noise. Permit multiple labels where
appropriate; never use real Reddit text.

Split about 60/40 into development and held-out sets, grouping threads/copies together to prevent leakage and keeping
themes represented on both sides. Freeze rules, thresholds and weights before held-out scoring. Keep the old scorer in
the evaluation harness as a frozen baseline; run old keywords, corrected keywords and semantic clustering on the same
inputs. Do not retain the old scorer on the production request path just for comparison.

Report weighted cluster purity **and** themed-item coverage, noise false-positive rate, and pairwise precision, recall
and F1 over independent stories. A pair is co-clustered when it shares any predicted category, accommodating the old
scorer's overlapping categories. Report multi-label handling explicitly. Purity alone rewards tiny clusters or rejecting
everything. Print raw counts and a confusion/example breakdown, not just one percentage.

Proposed acceptance targets: semantic pairwise F1 at least 0.15 above the legacy baseline, purity at least 0.85, themed
coverage at least 0.70 and noise false-positive rate at most 0.10 on the held-out fixture. These are unmeasured gates,
not results. Small synthetic sets cannot establish real-world accuracy; publish failures and review examples rather than
tuning against the held-out set. If necessary, revise the design with approval.

Add deterministic 1k/10k synthetic workload generation for performance tests; distinguish unique stories from duplicates
and disclose that synthetic timing is not a quality benchmark. Report hardware, window counts, cache-hit rate, cold/warm
runtime, peak RSS and actual Docker image size. Add regression tests for deterministic ties, copy/thread deduplication,
evidence ownership, cache invalidation, deletion/retention and quote exports. Run existing Rust, web, Compose, MCP and
offline-demo checks after each reviewable implementation increment.

## Approval scope and remaining uncertainties

Approval would authorize small Step B commits for shared grouping/ranking, keyword fixes, the cache migration, local
inference packaging, shared HTTP/MCP integration and additive UI/report fields, followed by Step C's evaluation fixture
and command. Preserve route/tool names, legacy defaults and old report rendering. Do not start Phase 4 in that work
unit.

The main uncertainties are actual CPU/image costs on supported architectures, the best distance threshold on this
domain, and how often English semantic similarity confuses neutral discussion with complaints. Small themes,
multilingual text, sarcasm and multiple themes in one thread remain known limits. The initial choice is MiniLM plus
average linkage and a keyword fallback; HDBSCAN, BGE and direct LLM providers require evidence before expanding that
scope.

## Primary references

Checked while preparing this proposal. Pin exact versions and artifact digests in the implementation.

[minilm]: https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2
[weights]: https://huggingface.co/Xenova/all-MiniLM-L6-v2/blob/08a308f628bc9d6774b7922f319eb1b65afa1a82/onnx/model_quantized.onnx
[bge]: https://huggingface.co/BAAI/bge-small-en-v1.5
[fastembed]: https://github.com/Anush008/fastembed-rs
[local-model]: https://docs.rs/fastembed/latest/fastembed/struct.UserDefinedEmbeddingModel.html
[kodama]: https://docs.rs/kodama/latest/kodama/
[hdbscan]: https://scikit-learn.org/stable/modules/generated/sklearn.cluster.HDBSCAN.html
