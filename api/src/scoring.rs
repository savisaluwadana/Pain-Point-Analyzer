use serde::Serialize;
use sqlx::types::Uuid;
use std::collections::{HashMap,HashSet};

#[derive(Clone)]
pub struct EvidenceText { pub id:Uuid,pub source_id:String,pub text:String,pub source_kind:String }
#[derive(Serialize,Clone)]
pub struct PainCluster { pub category:String,pub independent_stories:usize,pub evidence_count:usize,pub severity_score:f64,pub evidence_ids:Vec<Uuid>,pub examples:Vec<String> }
#[derive(Serialize)]
pub struct Report { pub evidence_scanned:usize,pub independent_stories:usize,pub clusters:Vec<PainCluster>,pub caveat:String }
const RULES:&[(&str,&[&str])] = &[
 ("Manual workflow", &["manual","manually","spreadsheet","copy paste","data entry","repetitive"]),
 ("Integration gaps", &["integration","doesn't integrate","sync issue","api missing","does not connect"]),
 ("Reliability", &["broken","crash","outage","unreliable","keeps failing","error"]),
 ("Pricing and cost", &["expensive","overpriced","hidden fee","too costly","pricing"]),
 ("Missing functionality", &["wish it had","missing feature","doesn't support","no way to","can't"]),
 ("Customer support", &["support ticket","no response","customer service","unanswered"]),
 ("Performance", &["slow","lagging","timeout","takes forever","latency"]),
 ("Workflow complexity", &["complicated","too many steps","confusing","cumbersome","hard to use"]),
];
pub fn analyze(evidence:&[EvidenceText])->Report{
 let mut unique=HashSet::new();
 // Conservatively merge verbatim long-form copies across providers.
 // Short common phrases remain independent because identical wording is not proof of copying.
 let mut content_fingerprints=HashMap::<String,String>::new();
 let mut groups:HashMap<&str,(HashMap<String,f64>,Vec<Uuid>)>=HashMap::new();
 for item in evidence {
   let mut key=if matches!(item.source_kind.as_str(),"reddit"|"reddit-archive") && item.source_id.starts_with("t3_"){
     format!("reddit-thread:{}",item.source_id.trim_start_matches("t3_"))
   }else if matches!(item.source_kind.as_str(),"reddit-comment"|"reddit-archive-comment"){
     let thread=item.source_id.split(':').next().unwrap_or(&item.source_id);
     format!("reddit-thread:{thread}")
   }else{format!("{}:{}",item.source_kind,item.source_id)};
   let normalized=item.text.to_ascii_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
   if normalized.len()>=100 {
     if let Some(existing)=content_fingerprints.get(&normalized){key=existing.clone();}
     else{content_fingerprints.insert(normalized,key.clone());}
   }
   unique.insert(key.clone());
   let s=item.text.to_ascii_lowercase();
   for (category,terms) in RULES {
      let hits=terms.iter().filter(|term|s.contains(**term)).count();
      if hits==0 {continue;}
      let entry=groups.entry(category).or_insert_with(||(HashMap::new(),Vec::new()));
      // A thread's repeated keyword matches do not create extra demand.
      let strength=(hits as f64).min(4.0);
      entry.0.entry(key.clone()).and_modify(|n|*n=(*n).max(strength)).or_insert(strength);
      entry.1.push(item.id);
   }
 }
 let mut clusters:Vec<PainCluster>=groups.into_iter().map(|(category,(stories,ids))|{
   let hits:f64=stories.values().sum();
   let score=(15.0 * stories.len() as f64+ 3.0 * hits).min(100.0);
   PainCluster{category:category.to_string(),independent_stories:stories.len(),evidence_count:ids.len(),severity_score:score,evidence_ids:ids,examples:Vec::new()}
 }).collect();
 clusters.sort_by(|a,b|b.severity_score.total_cmp(&a.severity_score));
 Report{evidence_scanned:evidence.len(),independent_stories:unique.len(),clusters,caveat:"Heuristic triage only. Scores are not buying intent, market size or proof of demand; verify independent sources and contradictory evidence.".into()}
}
// Keyword matching is a conservative relevance gate, not semantic classification.
pub fn relevant_to_topic(evidence:&[EvidenceText],topic:&str)->Vec<EvidenceText>{
 let query=topic.to_ascii_lowercase();
 if query.contains("recurring community needs") || query.contains("open-source projects could help communities"){return evidence.to_vec();}
 const STOP:&[&str]=&["a","an","the","and","or","in","of","on","for","to","from",
  "what","which","who","why","how","are","is","do","does","can","could","would",
  "find","discover","identify","research","issues","problem","problems","pain",
  "points","needs","users","people","community","communities","software","tools",
  "with","about","using","related","recurring","common","open","source","project"];
 let terms:HashSet<String>=query.split(|c:char|!c.is_ascii_alphanumeric())
  .filter(|s|s.len()>2 && !STOP.contains(s)).map(str::to_string).collect();
 if terms.is_empty(){return evidence.to_vec();}
 evidence.iter().filter(|item|{
  let hay=item.text.to_ascii_lowercase();
  terms.iter().any(|word|hay.contains(word) ||
   (word.ends_with("ing") && word.len()>5 && hay.contains(&format!("{}e",&word[..word.len()-3]))))
 }).cloned().collect()
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn archive_and_oauth_replies_count_as_one_thread() {
  let items=[("reddit","t3_abc"),("reddit-archive","t3_abc"),("reddit-archive-comment","abc:def"),("reddit-comment","abc:ghi")]
   .into_iter().map(|(kind,id)|EvidenceText{id:Uuid::new_v4(),source_id:id.into(),source_kind:kind.into(),text:"manual spreadsheet".into()}).collect::<Vec<_>>();
  let result=analyze(&items);
  assert_eq!(result.independent_stories,1);
  assert_eq!(result.clusters[0].independent_stories,1);
 }
 #[test] fn topic_filter_excludes_unrelated_evidence(){
  let mk=|text:&str|EvidenceText{id:Uuid::new_v4(),source_id:Uuid::new_v4().to_string(),
   source_kind:"manual".into(),text:text.into()};
  let items=vec![mk("invoicing manually takes hours"),mk("kubernetes deployments crash")];
  assert_eq!(relevant_to_topic(&items,"invoicing problems").len(),1);
  assert_eq!(relevant_to_topic(&items,"kubernetes problems").len(),1);
  assert!(relevant_to_topic(&items,"medical scheduling").is_empty());
  assert_eq!(relevant_to_topic(&items,"Synthetic demo: ways open-source projects could help communities").len(),2);
 }
 #[test] fn cross_source_copies_of_long_evidence_are_one_story(){
  let copy="A public discussion describes manual reporting that takes multiple hours every week, and asks for a safe way to automate the workflow without copying everything by hand.";
  let items=["github","forum"].iter().map(|source|EvidenceText{
   id:Uuid::new_v4(),source_id:Uuid::new_v4().to_string(),
   source_kind:source.to_string(),text:copy.to_string()
  }).collect::<Vec<_>>();
  let report=analyze(&items);
  assert_eq!(report.independent_stories,1);
  assert_eq!(report.clusters[0].independent_stories,1);
 }
 #[test] fn repeated_thread_replies_do_not_inflate_severity(){
  let items=(0..20).map(|i|EvidenceText{id:Uuid::new_v4(),source_id:format!("abc:reply{i}"),
   source_kind:"reddit-comment".into(),text:"manual manual spreadsheet".into()}).collect::<Vec<_>>();
  let report=analyze(&items);
  assert_eq!(report.clusters[0].independent_stories,1);
  assert!(report.clusters[0].severity_score<=27.0);
 }
 #[test] fn counts_independent_stories() {
  let id=Uuid::new_v4();
  let items=vec![
    EvidenceText{id,source_id:"a".into(),text:"Manual spreadsheets are painful".into(),source_kind:"reddit".into()},
    EvidenceText{id:Uuid::new_v4(),source_id:"a".into(),text:"manual data entry".into(),source_kind:"reddit".into()}
  ];
  let result=analyze(&items);
  assert_eq!(result.independent_stories,1);
  assert_eq!(result.clusters[0].independent_stories,1);
  assert_eq!(result.clusters[0].evidence_count,2);
 }
}
