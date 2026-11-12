//! Private per-OS-user SQLite storage, completely separate from the PostgreSQL web edition.
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Mutex, time::Duration};
use uuid::Uuid;

#[path = "../../../shared/scoring.rs"]
pub mod scoring;

pub struct DesktopDb(pub Mutex<Connection>);

#[derive(Serialize)]
pub struct Evidence {
    pub id: String,
    pub source_kind: String,
    pub source_id: String,
    pub source_url: String,
    pub title: String,
    pub body: String,
    pub created_at: String,
}
#[derive(Deserialize)]
pub struct NewEvidence {
    pub title: String,
    pub body: String,
    pub source_url: String,
}
#[derive(Serialize)]
pub struct ResearchRun {
    pub id: String,
    pub topic: String,
    pub report: serde_json::Value,
    pub created_at: String,
}
#[derive(Serialize)]
pub struct Opportunity {
    pub id: String,
    pub run_id: String,
    pub title: String,
    pub score: f64,
    pub independent_stories: usize,
    pub evidence_refs: Vec<String>,
    pub decision: String,
    pub existing_solutions: String,
    pub validation_notes: String,
    pub created_at: String,
}
#[derive(Serialize)]
pub struct Workspace {
    pub evidence: Vec<Evidence>,
    pub runs: Vec<ResearchRun>,
    pub opportunities: Vec<Opportunity>,
}

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS evidence (
  id TEXT PRIMARY KEY, source_kind TEXT NOT NULL,
  source_id TEXT NOT NULL UNIQUE, source_url TEXT NOT NULL DEFAULT '',
  title TEXT NOT NULL, body TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_desktop_evidence_date ON evidence(created_at DESC);
CREATE TABLE IF NOT EXISTS research_runs (
  id TEXT PRIMARY KEY, topic TEXT NOT NULL, report_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TABLE IF NOT EXISTS opportunities (
  id TEXT PRIMARY KEY, run_id TEXT NOT NULL REFERENCES research_runs(id) ON DELETE CASCADE,
  title TEXT NOT NULL, score REAL NOT NULL, independent_stories INTEGER NOT NULL,
  evidence_refs TEXT NOT NULL, decision TEXT NOT NULL DEFAULT 'unreviewed'
     CHECK(decision IN ('unreviewed','investigating','needs_validation','ready_to_build','not_pursuing')),
  existing_solutions TEXT NOT NULL DEFAULT '', validation_notes TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX IF NOT EXISTS idx_desktop_opportunity_date ON opportunities(created_at DESC);
PRAGMA user_version = 1;
"#;

pub fn open(path: &Path) -> Result<DesktopDb, String> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let connection = Connection::open(path).map_err(|e| e.to_string())?;
    connection.busy_timeout(Duration::from_secs(5)).map_err(|e| e.to_string())?;
    connection.execute_batch("PRAGMA journal_mode = WAL;").map_err(|e| e.to_string())?;
    let version: u32 = connection.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if version > 1 { return Err("This database comes from a newer desktop version; downgrade is blocked to protect your data".into()); }
    connection.execute_batch(SCHEMA).map_err(|e| e.to_string())?;
    Ok(DesktopDb(Mutex::new(connection)))
}

fn read_evidence(row: &rusqlite::Row<'_>) -> rusqlite::Result<Evidence> {
    Ok(Evidence { id:row.get(0)?, source_kind:row.get(1)?, source_id:row.get(2)?, source_url:row.get(3)?, title:row.get(4)?, body:row.get(5)?, created_at:row.get(6)? })
}
fn evidence(conn: &Connection, limit: i64) -> Result<Vec<Evidence>, String> {
    let mut q = conn.prepare("SELECT id,source_kind,source_id,source_url,title,body,created_at FROM evidence ORDER BY created_at DESC,id DESC LIMIT ?1").map_err(|e| e.to_string())?;
    let rows = q.query_map(params![limit], read_evidence).map_err(|e| e.to_string())?.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?;
    Ok(rows)
}
fn reports(conn: &Connection, limit: i64) -> Result<Vec<ResearchRun>,String> {
    let mut stmt=conn.prepare("SELECT id,topic,report_json,created_at FROM research_runs ORDER BY created_at DESC,id DESC LIMIT ?1").map_err(|e|e.to_string())?;
    let rows=stmt.query_map(params![limit], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).map_err(|e|e.to_string())?;
    rows.map(|r| {let (id,topic,raw,created_at)=r.map_err(|e|e.to_string())?;let report=serde_json::from_str(&raw).map_err(|e|e.to_string())?;Ok(ResearchRun{id,topic,report,created_at})}).collect()
}
fn opportunities(conn: &Connection, limit: i64) -> Result<Vec<Opportunity>,String> {
    let mut q=conn.prepare("SELECT id,run_id,title,score,independent_stories,evidence_refs,decision,existing_solutions,validation_notes,created_at FROM opportunities ORDER BY created_at DESC,id DESC LIMIT ?1").map_err(|e|e.to_string())?;
    let rows=q.query_map(params![limit], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,f64>(3)?,r.get::<_,i64>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?,r.get::<_,String>(7)?,r.get::<_,String>(8)?,r.get::<_,String>(9)?))).map_err(|e|e.to_string())?;
    rows.map(|r|{let (id,run_id,title,score,n,refs,decision,existing_solutions,validation_notes,created_at)=r.map_err(|e|e.to_string())?;let evidence_refs=serde_json::from_str(&refs).map_err(|e|e.to_string())?;Ok(Opportunity{id,run_id,title,score,independent_stories:n.max(0) as usize,evidence_refs,decision,existing_solutions,validation_notes,created_at})}).collect()
}
fn workspace_with_limits(conn: &Connection, evidence_limit: i64, run_limit: i64, opportunity_limit: i64) -> Result<Workspace,String> {
    Ok(Workspace{evidence:evidence(conn,evidence_limit)?,runs:reports(conn,run_limit)?,opportunities:opportunities(conn,opportunity_limit)?})
}
pub fn workspace(conn: &Connection) -> Result<Workspace,String> {
    // Limit the UI snapshot, never the data export.
    workspace_with_limits(conn,500,100,200)
}
fn valid_source_url(raw: &str) -> Result<(),String> {
    if raw.is_empty() {return Ok(());}
    if raw.len()>2048 {return Err("Source URL is too long".into());}
    let url=url::Url::parse(raw).map_err(|_|"Enter a valid HTTP or HTTPS URL")?;
    if !matches!(url.scheme(),"http"|"https") {return Err("Only HTTP and HTTPS source links are allowed".into());}
    let host=url.host_str().unwrap_or("").to_ascii_lowercase();
    let host=host.trim_end_matches('.');
    if matches!(host, "reddit.com"|"redd.it") || host.ends_with(".reddit.com") || host.ends_with(".redd.it") {
        return Err("Reddit content requires an approved provider connector; manual import is disabled".into());
    }
    Ok(())
}
pub fn save_evidence(conn:&Connection, entry: NewEvidence) -> Result<(),String> {
    let title=entry.title.trim(); let body=entry.body.trim(); let link=entry.source_url.trim();
    if title.is_empty()||title.chars().count()>500 {return Err("Evidence title must be 1–500 characters".into());}
    if body.is_empty()||body.chars().count()>10000 {return Err("Summarize the evidence in 1–10,000 characters".into());}
    valid_source_url(link)?;
    let id=Uuid::new_v4().to_string();
    conn.execute("INSERT INTO evidence(id,source_kind,source_id,source_url,title,body) VALUES(?1,'manual',?2,?3,?4,?5)",params![id,id,link,title,body]).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn remove_evidence(conn:&Connection,id:&str)->Result<(),String>{
    let changed=conn.execute("DELETE FROM evidence WHERE id=?1",[id]).map_err(|e|e.to_string())?;
    if changed==0 {return Err("Evidence was already removed".into());} Ok(())
}
const DEMO: &[(&str,&str)] = &[
("Manual project reporting","We manually copy paste weekly notes into spreadsheets and do repetitive data entry."),
("Performance regression","Our open-source dashboard is slow and takes forever to show results."),
("Integration missing","The contributor portal has a missing integration and cannot connect to our issue tracker."),
("Recurring support issue","Community support ignores requests and gives no response on a common blocker."),
("Confusing workflow","The deployment steps are complicated and hard to use for new contributors."),
("More manual reconciliation","Manual exports require us to manually reconcile records each week."),
];
pub fn seed_demo(conn:&mut Connection)->Result<usize,String>{
    let tx=conn.transaction().map_err(|e|e.to_string())?;
    let mut inserted=0;
    for (index,(title,body)) in DEMO.iter().enumerate() {
        inserted+=tx.execute("INSERT OR IGNORE INTO evidence(id,source_kind,source_id,source_url,title,body) VALUES(?1,'demo',?2,'',?3,?4)",params![Uuid::new_v4().to_string(),format!("fictional-demo-{}",index+1),title,body]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e|e.to_string())?; Ok(inserted)
}
pub fn analyze(conn:&mut Connection,topic:&str)->Result<(),String>{
    if topic.trim().is_empty()||topic.chars().count()>500 {return Err("Provide a research question (1–500 characters)".into());}
    let mut q=conn.prepare("SELECT id,source_kind,source_id,title,body FROM evidence ORDER BY created_at DESC,id DESC LIMIT 1000").map_err(|e|e.to_string())?;
    let samples=q.query_map([],|r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?))).map_err(|e|e.to_string())?;
    let mut items=Vec::new();
    for row in samples {let(id,kind,source_id,title,body)=row.map_err(|e|e.to_string())?;let uid=Uuid::parse_str(&id).map_err(|e|e.to_string())?;items.push(scoring::EvidenceText{id:uid,source_kind:kind,source_id,text:format!("{title}\n{body}")});}
    drop(q);
    let selected=scoring::relevant_to_topic(&items,topic);
    let report=scoring::analyze(&selected);
    let json=serde_json::to_string(&report).map_err(|e|e.to_string())?;
    let tx=conn.transaction().map_err(|e|e.to_string())?;
    let run_id=Uuid::new_v4().to_string();
    tx.execute("INSERT INTO research_runs(id,topic,report_json) VALUES(?1,?2,?3)",params![run_id,topic.trim(),json]).map_err(|e|e.to_string())?;
    for cluster in &report.clusters {
        tx.execute("INSERT INTO opportunities(id,run_id,title,score,independent_stories,evidence_refs) VALUES(?1,?2,?3,?4,?5,?6)",params![Uuid::new_v4().to_string(),run_id,cluster.category,cluster.severity_score,cluster.independent_stories as i64,serde_json::to_string(&cluster.evidence_ids).map_err(|e|e.to_string())?]).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e|e.to_string())?;Ok(())
}
pub fn save_review(conn:&Connection,id:&str,decision:&str,notes:&str,solutions:&str)->Result<(),String>{
    if !matches!(decision,"unreviewed"|"investigating"|"needs_validation"|"ready_to_build"|"not_pursuing") {return Err("Invalid decision".into());}
    if notes.chars().count()>4000||solutions.chars().count()>2000 {return Err("Review text is too long".into());}
    if decision=="ready_to_build" && notes.trim().chars().count()<20 {return Err("Ready to build requires 20+ characters of validation notes".into());}
    if conn.execute("UPDATE opportunities SET decision=?2,validation_notes=?3,existing_solutions=?4 WHERE id=?1",params![id,decision,notes.trim(),solutions.trim()]).map_err(|e|e.to_string())?==0 {return Err("Opportunity not found".into());}
    Ok(())
}

pub fn export_json(conn:&Connection,dir:&Path)->Result<String,String>{
    fs::create_dir_all(dir).map_err(|e|e.to_string())?;
    // Export the entire store rather than the bounded UI snapshot.
    let payload=serde_json::to_vec_pretty(&workspace_with_limits(conn,-1,-1,-1)?).map_err(|e|e.to_string())?;
    let destination=dir.join(format!("discovery-export-{}.json",Uuid::new_v4()));
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir,fs::Permissions::from_mode(0o700)).map_err(|e|e.to_string())?;
    }
    // Create a private file, and discard it if writing fails.
    #[cfg(unix)]
    let file={
        use std::os::unix::fs::OpenOptionsExt;
        fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&destination)
    };
    #[cfg(not(unix))]
    let file=fs::OpenOptions::new().write(true).create_new(true).open(&destination);
    use std::io::Write;
    let mut file=file.map_err(|e|e.to_string())?;
    if let Err(err)=file.write_all(&payload).and_then(|_|file.sync_all()) {
        drop(file);
        let _=fs::remove_file(&destination);
        return Err(err.to_string());
    }
    Ok(destination.to_string_lossy().into_owned())
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub evidence_imported: usize,
    pub evidence_skipped: usize,
    pub runs_imported: usize,
    pub runs_skipped: usize,
    pub opportunities_imported: usize,
    pub opportunities_skipped: usize,
}

pub fn import_json(conn: &mut Connection, raw: &str) -> Result<ImportSummary, String> {
    let root: serde_json::Value = serde_json::from_str(raw).map_err(|_| "Invalid JSON format")?;
    let mut summary = ImportSummary::default();

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. Evidence
    if let Some(items) = root.get("evidence").and_then(serde_json::Value::as_array) {
        for item in items {
            let title = item.get("title").and_then(serde_json::Value::as_str).unwrap_or("").trim();
            let body = item.get("body").and_then(serde_json::Value::as_str).unwrap_or("").trim();
            // Skip empty items (e.g. Reddit body excluded during web export for privacy)
            if title.is_empty() && body.is_empty() {
                summary.evidence_skipped += 1;
                continue;
            }
            let raw_url = item.get("source_url").and_then(serde_json::Value::as_str).unwrap_or("");
            if !raw_url.is_empty() && valid_source_url(raw_url).is_err() {
                summary.evidence_skipped += 1;
                continue;
            }
            let id = item.get("id").and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let source_kind = item.get("source_kind").and_then(serde_json::Value::as_str).unwrap_or("manual");
            let source_id = item.get("source_id").and_then(serde_json::Value::as_str)
                .unwrap_or(&id);

            // Check if record already exists by id or source_id
            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM evidence WHERE id = ?1 OR source_id = ?2)",
                params![&id, source_id],
                |r| r.get(0),
            ).unwrap_or(false);

            if exists {
                summary.evidence_skipped += 1;
            } else {
                let title_truncated: String = title.chars().take(300).collect();
                let body_truncated: String = body.chars().take(5000).collect();
                let created_at = item.get("created_at")
                    .or_else(|| item.get("collected_at"))
                    .and_then(serde_json::Value::as_str);

                let res = if let Some(created) = created_at {
                    tx.execute(
                        "INSERT INTO evidence(id, source_kind, source_id, source_url, title, body, created_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![&id, source_kind, source_id, raw_url, &title_truncated, &body_truncated, created],
                    )
                } else {
                    tx.execute(
                        "INSERT INTO evidence(id, source_kind, source_id, source_url, title, body) VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
                        params![&id, source_kind, source_id, raw_url, &title_truncated, &body_truncated],
                    )
                };

                match res {
                    Ok(_) => summary.evidence_imported += 1,
                    Err(_) => summary.evidence_skipped += 1,
                }
            }
        }
    }

    // 2. Research Runs (supports either "runs" from desktop or "research_runs" from web export)
    let runs_array = root.get("runs")
        .or_else(|| root.get("research_runs"))
        .and_then(serde_json::Value::as_array);

    if let Some(runs) = runs_array {
        for run in runs {
            let id = run.get("id").and_then(serde_json::Value::as_str);
            let topic = run.get("topic").and_then(serde_json::Value::as_str).unwrap_or("").trim();
            let report_val = run.get("report").or_else(|| run.get("report_json"));
            let Some(id) = id else {
                summary.runs_skipped += 1;
                continue;
            };
            let Some(report_val) = report_val else {
                summary.runs_skipped += 1;
                continue;
            };

            let report_json = if report_val.is_string() {
                report_val.as_str().unwrap().to_owned()
            } else {
                report_val.to_string()
            };

            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM research_runs WHERE id = ?1)",
                [id],
                |r| r.get(0),
            ).unwrap_or(false);

            if exists {
                summary.runs_skipped += 1;
            } else {
                let created_at = run.get("created_at").and_then(serde_json::Value::as_str);
                let res = if let Some(created) = created_at {
                    tx.execute(
                        "INSERT INTO research_runs(id, topic, report_json, created_at) VALUES(?1, ?2, ?3, ?4)",
                        params![id, topic, &report_json, created],
                    )
                } else {
                    tx.execute(
                        "INSERT INTO research_runs(id, topic, report_json) VALUES(?1, ?2, ?3)",
                        params![id, topic, &report_json],
                    )
                };
                match res {
                    Ok(_) => summary.runs_imported += 1,
                    Err(_) => summary.runs_skipped += 1,
                }
            }
        }
    }

    // 3. Opportunities
    if let Some(opps) = root.get("opportunities").and_then(serde_json::Value::as_array) {
        for opp in opps {
            let id = opp.get("id").and_then(serde_json::Value::as_str);
            let run_id = opp.get("run_id").and_then(serde_json::Value::as_str);
            let title = opp.get("title").and_then(serde_json::Value::as_str).unwrap_or("").trim();
            let Some(id) = id else {
                summary.opportunities_skipped += 1;
                continue;
            };
            let Some(run_id) = run_id else {
                summary.opportunities_skipped += 1;
                continue;
            };

            let score = opp.get("score").and_then(serde_json::Value::as_f64).unwrap_or(50.0);
            let stories = opp.get("independent_stories")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1) as i64;

            let refs_str = match opp.get("evidence_refs") {
                Some(r) if r.is_string() => r.as_str().unwrap().to_owned(),
                Some(r) => r.to_string(),
                None => "[]".to_owned(),
            };

            let raw_decision = opp.get("decision").and_then(serde_json::Value::as_str).unwrap_or("unreviewed");
            let decision = match raw_decision {
                "unreviewed" | "investigating" | "needs_validation" | "ready_to_build" | "not_pursuing" => raw_decision,
                _ => "unreviewed",
            };
            let solutions = opp.get("existing_solutions").and_then(serde_json::Value::as_str).unwrap_or("");
            let notes = opp.get("validation_notes").and_then(serde_json::Value::as_str).unwrap_or("");

            let exists: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM opportunities WHERE id = ?1)",
                [id],
                |r| r.get(0),
            ).unwrap_or(false);

            if exists {
                summary.opportunities_skipped += 1;
            } else {
                let run_exists: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM research_runs WHERE id = ?1)",
                    [run_id],
                    |r| r.get(0),
                ).unwrap_or(false);

                if !run_exists {
                    summary.opportunities_skipped += 1;
                    continue;
                }

                let created_at = opp.get("created_at").and_then(serde_json::Value::as_str);
                let res = if let Some(created) = created_at {
                    tx.execute(
                        "INSERT INTO opportunities(id, run_id, title, score, independent_stories, evidence_refs, decision, existing_solutions, validation_notes, created_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        params![id, run_id, title, score, stories, refs_str, decision, solutions, notes, created],
                    )
                } else {
                    tx.execute(
                        "INSERT INTO opportunities(id, run_id, title, score, independent_stories, evidence_refs, decision, existing_solutions, validation_notes) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![id, run_id, title, score, stories, refs_str, decision, solutions, notes],
                    )
                };
                match res {
                    Ok(_) => summary.opportunities_imported += 1,
                    Err(_) => summary.opportunities_skipped += 1,
                }
            }
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn db() -> Connection {let c=Connection::open_in_memory().unwrap();c.execute_batch(SCHEMA).unwrap();c}
    #[test] fn first_run_persists_demo_and_analysis(){
        let mut c=db();assert_eq!(seed_demo(&mut c).unwrap(),6);assert_eq!(seed_demo(&mut c).unwrap(),0);
        analyze(&mut c,"Synthetic demo: ways open-source projects could help communities").unwrap();
        let snapshot=workspace(&c).unwrap();assert_eq!(snapshot.evidence.len(),6);assert_eq!(snapshot.runs.len(),1);
        assert!(!snapshot.opportunities.is_empty());
        assert_eq!(snapshot.runs[0].report["engine"],"keywords-v2");
    }
    #[test] fn deny_unlicensed_and_script_links(){
        let c=db();for url in ["javascript:alert(1)","https://www.reddit.com/r/test","https://reddit.com/r/test","https://reddit.com./r/test","https://www.reddit.com./r/test","https://redd.it./abc"] {assert!(save_evidence(&c,NewEvidence{title:"test".into(),body:"manual crash".into(),source_url:url.into()}).is_err());}
        assert!(save_evidence(&c,NewEvidence{title:"normal".into(),body:"slow workflow".into(),source_url:"https://example.com".into()}).is_ok());
    }
    #[test] fn review_needs_justification(){let mut c=db();seed_demo(&mut c).unwrap();analyze(&mut c,"Synthetic demo: ways open-source projects could help communities").unwrap();let id=workspace(&c).unwrap().opportunities[0].id.clone();assert!(save_review(&c,&id,"ready_to_build","just a hunch","").is_err());assert!(save_review(&c,&id,"ready_to_build","Tested with three maintainers.","").is_ok());}

    #[test] fn export_includes_records_beyond_ui_preview_limits(){
        let c=db();
        for i in 0..505 {
            c.execute("INSERT INTO evidence(id,source_kind,source_id,source_url,title,body) VALUES(?1,'manual',?2,'','title','body')",
                params![format!("e-{i}"),format!("s-{i}")]).unwrap();
        }
        for i in 0..105 {
            c.execute("INSERT INTO research_runs(id,topic,report_json) VALUES(?1,'test','{}')",
                [format!("r-{i}")]).unwrap();
        }
        for i in 0..205 {
            c.execute("INSERT INTO opportunities(id,run_id,title,score,independent_stories,evidence_refs) VALUES(?1,'r-0','theme',50,1,'[]')",
                [format!("o-{i}")]).unwrap();
        }
        let preview=workspace(&c).unwrap();
        assert_eq!((preview.evidence.len(),preview.runs.len(),preview.opportunities.len()),(500,100,200));
        let dir=std::env::temp_dir().join(format!("discovery-export-test-{}",Uuid::new_v4()));
        let path=export_json(&c,&dir).unwrap();
        let archive:serde_json::Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(archive["evidence"].as_array().unwrap().len(),505);
        assert_eq!(archive["runs"].as_array().unwrap().len(),105);
        assert_eq!(archive["opportunities"].as_array().unwrap().len(),205);
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777,0o600);
            assert_eq!(fs::metadata(&dir).unwrap().permissions().mode() & 0o777,0o700);
        }
        fs::remove_dir_all(dir).unwrap();
    }
    #[test] fn removal_preserves_historical_counts(){let mut c=db();seed_demo(&mut c).unwrap();analyze(&mut c,"Synthetic demo: ways open-source projects could help communities").unwrap();let item=workspace(&c).unwrap().evidence[0].id.clone();remove_evidence(&c,&item).unwrap();assert_eq!(workspace(&c).unwrap().runs.len(),1);}

    #[test]
    fn import_web_postgresql_export_and_deduplicate() {
        let mut c = db();
        let web_export = r#"{
            "format": "pain-intelligence-lab-v1",
            "exported_at": "2026-11-12T00:00:00Z",
            "account": {"id": "user-1", "reddit_username": "researcher"},
            "projects": [{"id": "p1", "name": "Web Project", "description": "Test", "created_at": "2026-11-12T00:00:00Z"}],
            "evidence": [
                {
                    "id": "e-imported-1",
                    "source_kind": "github",
                    "source_id": "gh:100:1",
                    "source_url": "https://github.com/example/repo/issues/1",
                    "title": "Imported GitHub issue",
                    "body": "Slow deployment process",
                    "collected_at": "2026-11-12T00:00:00Z"
                },
                {
                    "id": "e-imported-2",
                    "source_kind": "reddit",
                    "source_id": "reddit:post:1",
                    "source_url": "",
                    "title": null,
                    "body": null,
                    "collected_at": "2026-11-12T00:00:00Z"
                }
            ],
            "research_runs": [
                {
                    "id": "r-imported-1",
                    "project_id": "p1",
                    "topic": "Imported deployment research",
                    "report": {"engine": "keywords-v2", "score_version": 2, "evidence_scanned": 1, "independent_stories": 1, "unclassified_items": 0, "clusters": [], "caveat": "Test"},
                    "created_at": "2026-11-12T00:00:00Z"
                }
            ],
            "opportunities": [
                {
                    "id": "o-imported-1",
                    "run_id": "r-imported-1",
                    "title": "Deployment Automation",
                    "score": 85.0,
                    "independent_stories": 1,
                    "evidence_refs": ["e-imported-1"],
                    "decision": "investigating",
                    "existing_solutions": "Manual bash scripts",
                    "validation_notes": "Spoke to team; reproduces consistently.",
                    "created_at": "2026-11-12T00:00:00Z"
                }
            ]
        }"#;

        let summary = import_json(&mut c, web_export).unwrap();
        assert_eq!(summary.evidence_imported, 1);
        assert_eq!(summary.evidence_skipped, 1);
        assert_eq!(summary.runs_imported, 1);
        assert_eq!(summary.opportunities_imported, 1);

        let ws = workspace(&c).unwrap();
        assert_eq!(ws.evidence.len(), 1);
        assert_eq!(ws.evidence[0].title, "Imported GitHub issue");
        assert_eq!(ws.evidence[0].source_kind, "github");
        assert_eq!(ws.runs.len(), 1);
        assert_eq!(ws.opportunities.len(), 1);
        assert_eq!(ws.opportunities[0].decision, "investigating");

        let summary2 = import_json(&mut c, web_export).unwrap();
        assert_eq!(summary2.evidence_imported, 0);
        assert_eq!(summary2.evidence_skipped, 2);
        assert_eq!(summary2.runs_imported, 0);
        assert_eq!(summary2.runs_skipped, 1);
        assert_eq!(summary2.opportunities_imported, 0);
        assert_eq!(summary2.opportunities_skipped, 1);
    }

    #[test]
    fn import_rejects_malformed_json_and_script_urls() {
        let mut c = db();
        assert!(import_json(&mut c, "not json").is_err());

        let malicious = r#"{
            "evidence": [{
                "id": "e-bad",
                "source_kind": "manual",
                "source_id": "bad-1",
                "source_url": "javascript:alert(1)",
                "title": "Bad Link",
                "body": "Malicious payload"
            }]
        }"#;
        let summary = import_json(&mut c, malicious).unwrap();
        assert_eq!(summary.evidence_imported, 0);
        assert_eq!(summary.evidence_skipped, 1);
    }
}
