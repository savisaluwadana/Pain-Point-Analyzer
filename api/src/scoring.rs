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
 let mut groups:HashMap<&str,(HashSet<String>,Vec<Uuid>,Vec<String>,f64)>=HashMap::new();
 for item in evidence {
   let key=if matches!(item.source_kind.as_str(),"reddit"|"reddit-archive") && item.source_id.starts_with("t3_"){
     format!("reddit-thread:{}",item.source_id.trim_start_matches("t3_"))
   }else if matches!(item.source_kind.as_str(),"reddit-comment"|"reddit-archive-comment"){
     let thread=item.source_id.split(':').next().unwrap_or(&item.source_id);
     format!("reddit-thread:{thread}")
   }else{format!("{}:{}",item.source_kind,item.source_id)};
   unique.insert(key.clone());
   let s=item.text.to_ascii_lowercase();
   for (category,terms) in RULES {
      let hits=terms.iter().filter(|term|s.contains(**term)).count();
      if hits==0 {continue;}
      let entry=groups.entry(category).or_insert_with(||(HashSet::new(),Vec::new(),Vec::new(),0.0));
      entry.0.insert(key.clone());
      entry.1.push(item.id);
      // Do not persist raw excerpts in long-lived aggregate reports.
      entry.3+=(hits as f64).min(4.0);
   }
 }
 let mut clusters:Vec<PainCluster>=groups.into_iter().map(|(category,(stories,ids,examples,hits))|{
   let score=(15.0 * stories.len() as f64+ 3.0 * hits).min(100.0);
   PainCluster{category:category.to_string(),independent_stories:stories.len(),evidence_count:ids.len(),severity_score:score,evidence_ids:ids,examples:Vec::new()}
 }).collect();
 clusters.sort_by(|a,b|b.severity_score.total_cmp(&a.severity_score));
 Report{evidence_scanned:evidence.len(),independent_stories:unique.len(),clusters,caveat:"Heuristic triage only. Scores are not buying intent, market size or proof of demand; verify independent sources and contradictory evidence.".into()}
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
