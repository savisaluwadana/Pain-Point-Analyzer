//! Dependency-free, auditable triage. This is not a semantic model or demand forecast.
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone)]
pub struct EvidenceText {
    pub id: Uuid,
    pub source_id: String,
    pub text: String,
    pub source_kind: String,
}
#[derive(Serialize, Clone)]
pub struct ExampleRef {
    pub evidence_id: Uuid,
    pub content_hash: String,
}
#[derive(Serialize, Clone)]
pub struct PainCluster {
    pub category: String,
    pub independent_stories: usize,
    pub evidence_count: usize,
    /// Compatibility alias: version 2 uses within-run relative priority, not severity.
    pub severity_score: f64,
    pub raw_priority: f64,
    pub source_families: usize,
    pub mean_complaint_strength: f64,
    pub sufficient_evidence: bool,
    pub evidence_ids: Vec<Uuid>,
    pub example_refs: Vec<ExampleRef>,
    /// Retained for old clients; never persist source text inside a report.
    pub examples: Vec<String>,
}
#[derive(Serialize)]
pub struct Report {
    pub engine: &'static str,
    pub score_version: u8,
    pub evidence_scanned: usize,
    pub independent_stories: usize,
    pub unclassified_items: usize,
    pub clusters: Vec<PainCluster>,
    pub caveat: String,
}
const RULES: &[(&str, &[&str])] = &[
    (
        "Manual workflow",
        &["manual", "manually", "copy paste", "data entry", "repetitive"],
    ),
    (
        "Integration gaps",
        &[
            "missing integration",
            "doesn't integrate",
            "sync issue",
            "api missing",
            "does not connect",
            "cannot connect",
        ],
    ),
    (
        "Reliability",
        &[
            "broken",
            "crash",
            "crashes",
            "outage",
            "unreliable",
            "keeps failing",
            "error",
            "errors",
        ],
    ),
    (
        "Pricing and cost",
        &["expensive", "overpriced", "hidden fee", "too costly", "cannot afford"],
    ),
    (
        "Missing functionality",
        &["wish it had", "missing feature", "doesn't support", "no way to"],
    ),
    (
        "Customer support",
        &["no response", "unanswered", "support ignores", "unhelpful support"],
    ),
    (
        "Performance",
        &[
            "slow",
            "lagging",
            "timeout",
            "times out",
            "takes forever",
            "high latency",
        ],
    ),
    (
        "Workflow complexity",
        &[
            "complicated",
            "too many steps",
            "confusing",
            "cumbersome",
            "hard to use",
        ],
    ),
];
fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .replace('’', "'")
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}
fn phrase_at(words: &[String], phrase: &[String], start: usize) -> bool {
    words.get(start..start + phrase.len()) == Some(phrase)
}
/// Clause boundaries prevent a negation in one clause from suppressing another.
/// This deliberately limited rule system does not claim to understand sarcasm.
fn positive_match(text: &str, term: &str) -> bool {
    let phrase = tokens(term);
    text.split(['.', ',', ';', '!', '?', '\n']).any(|clause| {
        let words = tokens(clause);
        (0..words.len()).any(|i| {
            if !phrase_at(&words, &phrase, i) {
                return false;
            }
            let boundary = words[..i]
                .iter()
                .rposition(|w| matches!(w.as_str(), "but" | "however" | "although"))
                .map_or(0, |j| j + 1);
            let start = boundary.max(i.saturating_sub(5));
            !(start..i).any(|j| {
                let negated = matches!(
                    words[j].as_str(),
                    "not"
                        | "no"
                        | "never"
                        | "without"
                        | "isn't"
                        | "aren't"
                        | "wasn't"
                        | "weren't"
                        | "don't"
                        | "doesn't"
                );
                negated && !(words[j] == "not" && words.get(j + 1).is_some_and(|w| w == "only"))
            })
        })
    })
}
pub fn content_hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}
/// Normalize provider-exposed parent IDs without inferring identity from authors.
pub fn analysis_source_id(kind: &str, original: &str, metadata: &serde_json::Value) -> String {
    let details = &metadata["details"];
    match kind {
        "hackernews" => details["story_id"]
            .as_i64()
            .filter(|id| *id > 0)
            .map(|id| format!("thread:{id}")),
        "discourse" => details["topic_id"].as_i64().filter(|id| *id > 0).and_then(|id| {
            details["forum"]
                .as_str()
                .filter(|host| !host.is_empty())
                .map(|host| format!("{host}:thread:{id}"))
        }),
        "youtube" => details["video_id"]
            .as_str()
            .filter(|id| !id.is_empty())
            .map(|id| format!("video:{id}")),
        _ => None,
    }
    .unwrap_or_else(|| original.to_owned())
}
fn story_key(item: &EvidenceText) -> String {
    if matches!(
        item.source_kind.as_str(),
        "reddit" | "reddit-archive" | "reddit-comment" | "reddit-archive-comment"
    ) {
        let thread = item
            .source_id
            .split(':')
            .next()
            .unwrap_or(&item.source_id)
            .trim_start_matches("t3_");
        format!("reddit-thread:{thread}")
    } else {
        format!("{}:{}", item.source_kind, item.source_id)
    }
}
fn root(parents: &mut [usize], mut i: usize) -> usize {
    while parents[i] != i {
        parents[i] = parents[parents[i]];
        i = parents[i];
    }
    i
}
fn unite(parents: &mut [usize], a: usize, b: usize) {
    let a = root(parents, a);
    let b = root(parents, b);
    parents[a.max(b)] = a.min(b);
}
pub fn analyze(evidence: &[EvidenceText]) -> Report {
    // Stable ordering plus union-find makes thread/copy grouping transitive and order-independent.
    let mut ordered: Vec<_> = evidence.iter().collect();
    ordered.sort_by_key(|e| e.id);
    let mut parents: Vec<_> = (0..ordered.len()).collect();
    let mut threads = HashMap::new();
    let mut copies = HashMap::new();
    for (i, item) in ordered.iter().enumerate() {
        if let Some(j) = threads.insert(story_key(item), i) {
            unite(&mut parents, i, j);
        }
        let normalized = item
            .text
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if normalized.chars().count() >= 100 {
            if let Some(j) = copies.insert(normalized, i) {
                unite(&mut parents, i, j);
            }
        }
    }
    let keys: Vec<_> = (0..ordered.len()).map(|i| root(&mut parents, i)).collect();
    let independent = keys.iter().copied().collect::<BTreeSet<_>>().len();
    let mut classified = BTreeSet::new();
    let mut clusters = Vec::new();
    for (category, terms) in RULES {
        let mut stories: BTreeMap<usize, (f64, &EvidenceText)> = BTreeMap::new();
        let mut ids = Vec::new();
        for (i, item) in ordered.iter().enumerate() {
            let hits = terms.iter().filter(|term| positive_match(&item.text, term)).count();
            if hits == 0 {
                continue;
            }
            classified.insert(item.id);
            ids.push(item.id);
            let strength = hits.min(4) as f64 / 4.0;
            let entry = stories.entry(keys[i]).or_insert((strength, item));
            entry.0 = entry.0.max(strength);
        }
        if stories.is_empty() {
            continue;
        }
        // Assign each copied story one canonical provider; reposts never inflate diversity.
        let families = stories
            .keys()
            .map(|&key| {
                let kind = &ordered[key].source_kind;
                if kind.starts_with("reddit") {
                    "reddit"
                } else {
                    kind.as_str()
                }
            })
            .collect::<BTreeSet<_>>()
            .len();
        let strength = stories.values().map(|(s, _)| s).sum::<f64>() / stories.len() as f64;
        let raw_priority = 0.7 * (stories.len() as f64).ln_1p() + 0.15 * (families as f64).ln_1p() + 0.15 * strength;
        clusters.push(PainCluster {
            category: (*category).into(),
            independent_stories: stories.len(),
            evidence_count: ids.len(),
            severity_score: 50.0,
            raw_priority,
            source_families: families,
            mean_complaint_strength: strength,
            sufficient_evidence: stories.len() >= 3,
            evidence_ids: ids,
            example_refs: stories
                .values()
                .take(5)
                .map(|(_, e)| ExampleRef {
                    evidence_id: e.id,
                    content_hash: content_hash(&e.text),
                })
                .collect(),
            examples: Vec::new(),
        });
    }
    let scores: Vec<_> = clusters.iter().map(|c| c.raw_priority).collect();
    for cluster in &mut clusters {
        if scores.len() > 1 {
            let below = scores.iter().filter(|&&s| s < cluster.raw_priority).count() as f64;
            let equal = scores.iter().filter(|&&s| s == cluster.raw_priority).count() as f64;
            cluster.severity_score = 100.0 * (below + (equal - 1.0) / 2.0) / (scores.len() - 1) as f64;
        }
    }
    clusters.sort_by(|a, b| {
        b.raw_priority
            .total_cmp(&a.raw_priority)
            .then(a.category.cmp(&b.category))
    });
    Report {
        engine: "keywords-v2", score_version: 2, evidence_scanned: evidence.len(),
        independent_stories: independent, unclassified_items: evidence.len().saturating_sub(classified.len()), clusters,
        caveat: "Rule-based hypotheses, not verified demand. Relative priority compares themes only within this run;             raw priority grows with independent support. Fewer than three stories is insufficient recurring evidence.             Negation and paraphrase handling remain limited; read original sources and contradictory evidence.".into(),
    }
}
pub fn topic_terms(topic: &str) -> Vec<String> {
    let query = topic.to_ascii_lowercase();
    if query.contains("recurring community needs") || query.contains("open-source projects could help communities") {
        return Vec::new();
    }
    const STOP: &[&str] = &[
        "a",
        "an",
        "the",
        "and",
        "or",
        "in",
        "of",
        "on",
        "for",
        "to",
        "from",
        "what",
        "which",
        "who",
        "why",
        "how",
        "are",
        "is",
        "do",
        "does",
        "can",
        "could",
        "would",
        "find",
        "discover",
        "identify",
        "research",
        "issues",
        "problem",
        "problems",
        "pain",
        "points",
        "needs",
        "users",
        "people",
        "community",
        "communities",
        "software",
        "tools",
        "with",
        "about",
        "using",
        "related",
        "recurring",
        "common",
        "open",
        "source",
        "project",
    ];
    let mut terms: Vec<String> = query
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| s.len() > 2 && !STOP.contains(s))
        .map(str::to_string)
        .collect();
    terms.sort();
    terms.dedup();
    terms
}
pub fn relevant_to_topic(evidence: &[EvidenceText], topic: &str) -> Vec<EvidenceText> {
    let terms = topic_terms(topic);
    if terms.is_empty() {
        return evidence.to_vec();
    }
    evidence
        .iter()
        .filter(|item| {
            let words = tokens(&item.text);
            terms.iter().any(|word| {
                words.contains(word)
                    || (word.ends_with("ing")
                        && word.len() > 5
                        && words.contains(&format!("{}e", &word[..word.len() - 3])))
            })
        })
        .cloned()
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_and_oauth_replies_count_as_one_thread() {
        let items = [
            ("reddit", "t3_abc"),
            ("reddit-archive", "t3_abc"),
            ("reddit-archive-comment", "abc:def"),
            ("reddit-comment", "abc:ghi"),
        ]
        .into_iter()
        .map(|(kind, id)| EvidenceText {
            id: Uuid::new_v4(),
            source_id: id.into(),
            source_kind: kind.into(),
            text: "manual spreadsheet".into(),
        })
        .collect::<Vec<_>>();
        let result = analyze(&items);
        assert_eq!(result.independent_stories, 1);
        assert_eq!(result.clusters[0].independent_stories, 1);
    }
    #[test]
    fn topic_filter_excludes_unrelated_evidence() {
        let mk = |text: &str| EvidenceText {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4().to_string(),
            source_kind: "manual".into(),
            text: text.into(),
        };
        let items = vec![mk("invoicing manually takes hours"), mk("kubernetes deployments crash")];
        assert_eq!(relevant_to_topic(&items, "invoicing problems").len(), 1);
        assert_eq!(relevant_to_topic(&items, "kubernetes problems").len(), 1);
        assert!(relevant_to_topic(&items, "medical scheduling").is_empty());
        assert_eq!(
            relevant_to_topic(
                &items,
                "Synthetic demo: ways open-source projects could help communities"
            )
            .len(),
            2
        );
    }
    #[test]
    fn cross_source_copies_of_long_evidence_are_one_story() {
        let copy = "A public discussion describes manual reporting that takes multiple hours every week, and asks \
            for a safe way to automate the workflow without copying everything by hand.";
        let items = ["github", "forum"]
            .iter()
            .map(|source| EvidenceText {
                id: Uuid::new_v4(),
                source_id: Uuid::new_v4().to_string(),
                source_kind: source.to_string(),
                text: copy.to_string(),
            })
            .collect::<Vec<_>>();
        let report = analyze(&items);
        assert_eq!(report.independent_stories, 1);
        assert_eq!(report.clusters[0].independent_stories, 1);
    }
    #[test]
    fn repeated_thread_replies_do_not_inflate_severity() {
        let items = (0..20)
            .map(|i| EvidenceText {
                id: Uuid::new_v4(),
                source_id: format!("abc:reply{i}"),
                source_kind: "reddit-comment".into(),
                text: "manual manual spreadsheet".into(),
            })
            .collect::<Vec<_>>();
        let report = analyze(&items);
        assert_eq!(report.clusters[0].independent_stories, 1);
        assert_eq!(report.clusters[0].severity_score, 50.0);
    }
    #[test]
    fn counts_independent_stories() {
        let id = Uuid::new_v4();
        let items = vec![
            EvidenceText {
                id,
                source_id: "a".into(),
                text: "Manual spreadsheets are painful".into(),
                source_kind: "reddit".into(),
            },
            EvidenceText {
                id: Uuid::new_v4(),
                source_id: "a".into(),
                text: "manual data entry".into(),
                source_kind: "reddit".into(),
            },
        ];
        let result = analyze(&items);
        assert_eq!(result.independent_stories, 1);
        assert_eq!(result.clusters[0].independent_stories, 1);
        assert_eq!(result.clusters[0].evidence_count, 2);
    }
    fn sample(i: u128, text: &str) -> EvidenceText {
        EvidenceText {
            id: Uuid::from_u128(i),
            source_id: i.to_string(),
            source_kind: "manual".into(),
            text: text.into(),
        }
    }
    #[test]
    fn neutral_substrings_and_negations_are_not_complaints() {
        for text in [
            "terror",
            "pricing guide",
            "integration documentation",
            "I can't praise this enough",
            "not slow",
            "no errors",
            "never crashes",
            "without complicated setup",
            "isn’t expensive",
        ] {
            assert!(analyze(&[sample(1, text)]).clusters.is_empty(), "{text}");
        }
        let report = analyze(&[sample(1, "not slow, but expensive")]);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.clusters[0].category, "Pricing and cost");
        assert!(positive_match("not only slow but also expensive", "slow"));
        assert!(positive_match("no response from support", "no response"));
        assert!(positive_match("doesn’t support keyboard access", "doesn't support"));
    }
    #[test]
    fn transitive_copies_and_threads_are_stable_when_shuffled() {
        let copy = "manual reporting ".repeat(10);
        let a = sample(1, &copy);
        let mut b = sample(2, &copy);
        b.source_kind = "reddit-comment".into();
        b.source_id = "thread:b".into();
        let mut c = sample(3, "manual work");
        c.source_kind = "reddit-comment".into();
        c.source_id = "thread:c".into();
        let expected = analyze(&[a.clone(), b.clone(), c.clone()]);
        assert_eq!(expected.independent_stories, 1);
        assert_eq!(expected.clusters[0].independent_stories, 1);
        assert_eq!(
            serde_json::to_value(&expected).unwrap(),
            serde_json::to_value(analyze(&[c, b, a])).unwrap()
        );
    }
    #[test]
    fn raw_ranking_does_not_saturate_and_reports_do_not_copy_source_text() {
        let small = (1..7).map(|i| sample(i, "slow")).collect::<Vec<_>>();
        let large = (1..101).map(|i| sample(i, "slow")).collect::<Vec<_>>();
        let a = analyze(&small);
        let b = analyze(&large);
        assert!(b.clusters[0].raw_priority > a.clusters[0].raw_priority);
        assert_eq!(a.clusters[0].severity_score, 50.0);
        assert!(a.clusters[0].examples.is_empty());
        assert_eq!(b.clusters[0].example_refs.len(), 5);
        let secret = "slow private sentence should never be copied into a historical report";
        assert!(!serde_json::to_string(&analyze(&[sample(1, secret)]))
            .unwrap()
            .contains(secret));
    }
    #[test]
    fn ties_are_ordered_and_sparse_support_is_explicit() {
        let report = analyze(&[sample(1, "slow expensive"), sample(2, "neutral discussion")]);
        assert_eq!(report.unclassified_items, 1);
        assert_eq!(report.clusters[0].category, "Performance");
        assert_eq!(report.clusters[0].severity_score, report.clusters[1].severity_score);
        assert!(!report.clusters[0].sufficient_evidence);
    }
    #[test]
    fn provider_parent_groups_keep_forums_separate() {
        use serde_json::json;
        assert_eq!(
            analysis_source_id("hackernews", "comment", &json!({"details":{"story_id":42}})),
            "thread:42"
        );
        assert_ne!(
            analysis_source_id("discourse", "1", &json!({"details":{"topic_id":1,"forum":"a.example"}})),
            analysis_source_id("discourse", "1", &json!({"details":{"topic_id":1,"forum":"b.example"}}))
        );
        assert_eq!(analysis_source_id("hackernews", "comment", &json!({})), "comment");
    }
    #[test]
    #[ignore = "Opt-in deterministic workload timing; not a quality benchmark"]
    fn analysis_workload_benchmark() {
        for count in [1000, 10000] {
            let records = (1..=count).map(|i| sample(i, &format!(
                "Fictional workflow {i}: manual reconciliation is slow and expensive. We need clearer exports for this invented scenario.")))
                .collect::<Vec<_>>();
            let start = std::time::Instant::now();
            let report = analyze(&records);
            eprintln!(
                "keywords-v2: {} unique records, {} ms, {} clusters",
                count,
                start.elapsed().as_millis(),
                report.clusters.len()
            );
            assert_eq!(report.independent_stories, count as usize);
            assert_eq!(report.evidence_scanned, count as usize);
        }
    }
}
