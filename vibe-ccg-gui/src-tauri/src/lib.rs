use tauri::{AppHandle, State, Emitter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::process::Stdio;
use tokio::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use uuid::Uuid;
use chrono::Utc;

pub mod db;
pub mod orchestrator;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowRun {
    pub id: String,
    pub workspace_id: String,
    pub status: String,
    pub mode: Option<String>,
    pub prompt: Option<String>,
    pub logs_path: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

// Global App State
pub struct AppState {
    pub active_processes: Arc<Mutex<HashMap<String, bool>>>,
}

#[derive(Clone, Serialize)]
struct LogPayload {
    line: String,
}

#[tauri::command]
fn get_workspaces(app: tauri::AppHandle) -> Result<Vec<Workspace>, String> {
    db::get_workspaces(&app).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_workspace(app: tauri::AppHandle, name: String, path: String) -> Result<Workspace, String> {
    let ws = Workspace {
        id: Uuid::new_v4().to_string(),
        name,
        path,
        created_at: Some(Utc::now().to_rfc3339()),
    };
    db::insert_workspace(&app, &ws).map_err(|e| e.to_string())?;
    Ok(ws)
}

#[tauri::command]
fn get_workflow_runs(app: tauri::AppHandle, workspace_id: String) -> Result<Vec<WorkflowRun>, String> {
    db::get_workflow_runs(&app, &workspace_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        tauri_plugin_sql::Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "
                CREATE TABLE IF NOT EXISTS workspaces (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    path TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );
                CREATE TABLE IF NOT EXISTS workflow_runs (
                    id TEXT PRIMARY KEY,
                    workspace_id TEXT NOT NULL,
                    mode TEXT,
                    prompt TEXT,
                    status TEXT NOT NULL,
                    logs_path TEXT,
                    start_time DATETIME DEFAULT CURRENT_TIMESTAMP,
                    end_time DATETIME,
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
                );
            ",
            kind: tauri_plugin_sql::MigrationKind::Up,
        },
        tauri_plugin_sql::Migration {
            version: 2,
            description: "create_workflow_steps",
            sql: "
                CREATE TABLE IF NOT EXISTS workflow_steps (
                    id TEXT PRIMARY KEY,
                    run_id TEXT NOT NULL,
                    step_index INTEGER NOT NULL,
                    step_name TEXT NOT NULL,
                    status TEXT NOT NULL DEFAULT 'pending',
                    input_prompt TEXT,
                    output_text TEXT,
                    output_summary TEXT,
                    session_id TEXT,
                    codex_session TEXT,
                    gemini_session TEXT,
                    start_time DATETIME,
                    end_time DATETIME,
                    error_message TEXT,
                    FOREIGN KEY (run_id) REFERENCES workflow_runs(id)
                );
            ",
            kind: tauri_plugin_sql::MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .manage(AppState {
            active_processes: Arc::new(Mutex::new(HashMap::new())),
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().add_migrations("sqlite:vibe-ccg.db", migrations).build())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_workspaces,
            add_workspace,
            get_workflow_runs,
            orchestrator::start_workflow_run,
            orchestrator::execute_step,
            orchestrator::retry_step,
            orchestrator::get_run_steps,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
