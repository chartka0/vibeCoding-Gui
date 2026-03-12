use tauri::{AppHandle, State, Emitter};
use tauri_plugin_sql::{Migration, MigrationKind};
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
    pub logs_path: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

// Global App State to hold active processes
pub struct AppState {
    // workspace_id -> Process Handle (or just status for now)
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
async fn start_workflow(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<String, String> {
    {
        let processes = state.active_processes.lock().await;
        if processes.contains_key(&workspace_id) {
            return Err("A workflow is already running for this workspace".into());
        }
    }

    let workspaces = db::get_workspaces(&app).map_err(|e| e.to_string())?;
    let workspace = workspaces.into_iter().find(|w| w.id == workspace_id).ok_or("Workspace not found")?;

    let run_id = Uuid::new_v4().to_string();
    let run = WorkflowRun {
        id: run_id.clone(),
        workspace_id: workspace_id.clone(),
        status: "Running".to_string(),
        logs_path: None,
        start_time: Some(Utc::now().to_rfc3339()),
        end_time: None,
    };
    db::create_workflow_run(&app, &run).map_err(|e| e.to_string())?;

    {
        let mut processes = state.active_processes.lock().await;
        processes.insert(workspace_id.clone(), true);
    }

    let app_clone = app.clone();
    let state_clone = state.inner().active_processes.clone();
    let run_id_clone = run_id.clone();
    let workspace_id_clone = workspace_id.clone();
    let workspace_path = workspace.path.clone();

    tokio::spawn(async move {
        let mut cmd = AsyncCommand::new("ccg-workflow");
        cmd.current_dir(&workspace_path)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child_res = cmd.spawn();
        
        // Fallback mock for development if ccg-workflow is not installed
        if child_res.is_err() {
            #[cfg(debug_assertions)]
            {
                let mut mock_cmd = AsyncCommand::new(if cfg!(windows) { "cmd" } else { "sh" });
                mock_cmd.arg(if cfg!(windows) { "/C" } else { "-c" });
                
                let mock_script = if cfg!(windows) {
                    "echo Starting vibe-ccg workflow for local project... && timeout 2 > NUL && echo Scanning AST and resolving dependencies... && timeout 2 > NUL && echo Building phase complete. Success!"
                } else {
                    "echo Starting vibe-ccg workflow for local project... && sleep 2 && echo Scanning AST and resolving dependencies... && sleep 2 && echo Building phase complete. Success!"
                };
                
                mock_cmd.arg(mock_script);
                mock_cmd.current_dir(&workspace_path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                child_res = mock_cmd.spawn();
            }
        }

        let mut child = match child_res {
            Ok(c) => c,
            Err(e) => {
                let _ = app_clone.emit(&format!("workflow-log-{}", workspace_id_clone), LogPayload { line: format!("Failed to start process: {}", e) });
                let _ = db::update_workflow_run_status(&app_clone, &run_id_clone, "Failed", &Utc::now().to_rfc3339(), None);
                let mut processes = state_clone.lock().await;
                processes.remove(&workspace_id_clone);
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            let event_name = format!("workflow-log-{}", workspace_id_clone);
            while let Ok(Some(line)) = reader.next_line().await {
                // Ignore empty bytes in windows timeout output if any
                if !line.trim().is_empty() {
                    let _ = app_clone.emit(&event_name, LogPayload { line });
                }
            }
        }

        let status = child.wait().await;
        let final_status = match status {
            Ok(s) if s.success() => "Success",
            _ => "Failed",
        };

        let _ = db::update_workflow_run_status(&app_clone, &run_id_clone, final_status, &Utc::now().to_rfc3339(), None);

        let mut processes = state_clone.lock().await;
        processes.remove(&workspace_id_clone);
        
        // Emit completion event
        let _ = app_clone.emit(&format!("workflow-done-{}", workspace_id_clone), ());
    });

    Ok(run_id)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
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
                    status TEXT NOT NULL,
                    logs_path TEXT,
                    start_time DATETIME DEFAULT CURRENT_TIMESTAMP,
                    end_time DATETIME,
                    FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
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
            start_workflow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
