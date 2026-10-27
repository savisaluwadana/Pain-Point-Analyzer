//! A private single-user native application: no HTTP listener, no hosted auth or API keys.
mod db;
use std::{path::PathBuf, sync::MutexGuard};
use tauri::{Manager, State};

fn guard<'a>(state:&'a State<'_,db::DesktopDb>)->Result<MutexGuard<'a,rusqlite::Connection>,String>{
    state.0.lock().map_err(|_| "Database is temporarily unavailable".into())
}
#[tauri::command]
fn load_workspace(state:State<'_,db::DesktopDb>)->Result<db::Workspace,String>{
    db::workspace(&*guard(&state)?)
}
#[tauri::command]
fn save_evidence(state:State<'_,db::DesktopDb>,entry:db::NewEvidence)->Result<(),String>{
    db::save_evidence(&*guard(&state)?,entry)
}
#[tauri::command]
fn remove_evidence(state:State<'_,db::DesktopDb>,id:String)->Result<(),String>{
    db::remove_evidence(&*guard(&state)?,&id)
}
#[tauri::command]
fn seed_demo(state:State<'_,db::DesktopDb>)->Result<usize,String>{
    db::seed_demo(&mut *guard(&state)?)
}
#[tauri::command]
fn run_analysis(state:State<'_,db::DesktopDb>,topic:String)->Result<(),String>{
    db::analyze(&mut *guard(&state)?,&topic)
}
#[tauri::command]
fn save_review(state:State<'_,db::DesktopDb>,id:String,decision:String,notes:String,solutions:String)->Result<(),String>{
    db::save_review(&*guard(&state)?,&id,&decision,&notes,&solutions)
}
#[tauri::command]
fn export_workspace(app:tauri::AppHandle,state:State<'_,db::DesktopDb>)->Result<String,String>{
    let path:PathBuf=app.path().app_data_dir().map_err(|e|e.to_string())?.join("exports");
    db::export_json(&*guard(&state)?,&path)
}

pub fn run(){
    tauri::Builder::default()
        .setup(|app|{
            let dir=app.path().app_data_dir()?;
            let database=db::open(&dir.join("research.sqlite3"))
                .map_err(std::io::Error::other)?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_workspace,save_evidence,remove_evidence,seed_demo,run_analysis,save_review,export_workspace])
        .run(tauri::generate_context!())
        .expect("Unable to start local Open Source Discovery Lab");
}
