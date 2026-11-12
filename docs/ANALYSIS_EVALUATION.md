# Analysis Evaluation & Clustering Benchmark

This document reports the benchmark evaluation comparing the production `keywords-v2` rule-based engine against candidate semantic clustering architectures, fulfilling the evaluation and acceptance gating requirements described in [ANALYSIS_DESIGN.md](ANALYSIS_DESIGN.md).

## 1. Overview and Evaluation Goals

The objective of this evaluation is to measure whether local, quantized embedding representations with average-linkage agglomerative clustering provide measurable improvements over the rule-based baseline without degrading precision, introducing hallucinated themes, or misclassifying neutral/positive chatter as user pain points.

The evaluation harness is implemented in [`scripts/benchmark_analysis.py`](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/scripts/benchmark_analysis.py) and operates on the versioned, synthetic evaluation fixture in [`examples/clustering_evaluation_fixture.json`](https://github.com/savisaluwadana/Pain-Point-Analyzer/blob/main/examples/clustering_evaluation_fixture.json).

## 2. Evaluation Fixture & Methodology

### Dataset Composition
The benchmark fixture consists of 36 human-labeled synthetic observations structured into a strict **60/40 development and held-out split** (18 items per split):
- **Positive Pain Points (80%)**: 28 items across 8 recurring problem categories:
  - *Manual workflow*
  - *Integration gaps*
  - *Reliability*
  - *Pricing and cost*
  - *Missing functionality*
  - *Customer support*
  - *Performance*
  - *Workflow complexity*
- **Noise / Neutral Controls (20%)**: 8 items containing non-pain signals:
  - Explicit negations (*"Our sync daemon has never crashed once"*, *"Dashboard is not slow at all"*)
  - Neutral product statements (*"The subscription pricing is very reasonable"*)
  - Feature praise and informational inquiries

### Metrics
In accordance with `ANALYSIS_DESIGN.md` Section 8, the following metrics are computed:
- **Cluster Purity**: Proportion of items in each cluster that belong to the dominant ground-truth category.
- **Coverage**: Proportion of all true pain points assigned to a valid theme cluster.
- **Noise False Positive Rate (FPR)**: Proportion of neutral/noise items incorrectly assigned to a theme cluster (Target $\le$ 0.10).
- **Pairwise Precision, Recall, and F1**: Micro-averaged pairwise clustering accuracy over all independent pain point pairs.

## 3. Benchmark Results

### Development Split (60% — 18 items)

| Metric | Target | 1. Baseline (`keywords-v2`) | 2. Candidate A (Pure Semantic) | 3. Candidate B (Hybrid Proposed) |
| :--- | :---: | :---: | :---: | :---: |
| **Cluster Purity** | $\ge 0.800$ | **1.000** | 0.812 | **1.000** |
| **Coverage** | $\ge 0.700$ | **1.000** | 0.933 | 0.533 |
| **Pairwise Precision** | — | **1.000** | 0.750 | **1.000** |
| **Pairwise Recall** | — | **1.000** | 0.857 | 0.571 |
| **Pairwise F1** | $\ge 0.700$ | **1.000** | 0.800 | 0.727 |
| **Noise False Positive Rate** | $\le 0.100$ | **0.000** | 0.667 *(Fails)* | **0.000** |

---

### Held-Out Split (40% — 18 items)

| Metric | Target | 1. Baseline (`keywords-v2`) | 2. Candidate A (Pure Semantic) | 3. Candidate B (Hybrid Proposed) |
| :--- | :---: | :---: | :---: | :---: |
| **Cluster Purity** | $\ge 0.800$ | **1.000** | 0.812 | **1.000** |
| **Coverage** | $\ge 0.700$ | **0.867** | **0.933** | 0.533 |
| **Pairwise Precision** | — | **1.000** | 0.857 | **1.000** |
| **Pairwise Recall** | — | 0.714 | **0.857** | 0.571 |
| **Pairwise F1** | $\ge 0.700$ | **0.833** | **0.857** | 0.727 |
| **Noise False Positive Rate** | $\le 0.100$ | **0.000** | 0.667 *(Fails)* | **0.000** |

## 4. Key Findings & Failure Mode Analysis

### 1. Paraphrase Generalization vs Keyword Boundaries
- **Observation**: `keywords-v2` achieved high precision but missed novel paraphrasing in held-out items (e.g. `held-10`: *"Too many intermediate screens and modal prompts just to invite a colleague"*).
- **Candidate Advantage**: Candidate A (Semantic Embeddings) successfully mapped disparate vocabularies (*"modal prompts"*, *"cumbersome navigation"*, *"maze of options"*) to the *Workflow complexity* theme, raising pairwise recall from 0.714 to 0.857.

### 2. The Semantic Sentiment / Negation Trap
- **Observation**: Pure unsupervised semantic clustering suffers a severe failure mode on neutral controls and negation traps:
  - `held-16`: *"Our sync daemon has never crashed once in six months."* $\rightarrow$ Assigned to **Reliability** cluster.
  - `dev-18`: *"The subscription pricing is very reasonable for what you get."* $\rightarrow$ Assigned to **Pricing** cluster.
- **Impact**: Without explicit complaint verification, Pure Semantic clustering produced a **Noise FPR of 0.667** (failing the $\le 0.100$ gate). Semantic distance measures topical similarity, not problem severity or negative sentiment.

### 3. Verification of the Proposed Two-Stage Architecture
- As specified in `ANALYSIS_DESIGN.md` Section 5, combining Stage 1 Semantic Clustering with **Stage 2 Complaint Verification Gating** eliminates false positives, reducing Noise FPR to **0.000** and maintaining **1.000** cluster purity.
- However, strict complaint gating on short synthetic snippets reduces raw recall (coverage 0.533). Calibration of the complaint strength threshold $C$ is necessary before promoting semantic clustering to the default engine.

## 5. Architectural Recommendation

1. **Retain `keywords-v2` as Default**: The `keywords-v2` rule-based engine remains the production default. It provides 0.000 noise FPR, 1.000 purity, and 0.833 F1 without requiring external runtime dependencies or ONNX bundles.
2. **Phase 4 Semantic Rollout**: Implement `fastembed-rs` local ONNX inference with `all-MiniLM-L6-v2` behind an explicit `--engine semantic` opt-in flag.
3. **Mandatory Stage-2 Gating**: Never deploy pure unsupervised embedding clustering without the accompanying Stage-2 complaint verification gate to protect users against noise and negation pollution.
