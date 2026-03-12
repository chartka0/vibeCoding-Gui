use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use tauri::Manager;
use crate::{Workspace, WorkflowRun};

// Helper function to get the database path
pub fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_dir = app_handle.path().app_local_data_dir().expect("Failed to get app local data dir");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
    app_dir.join("vibe-ccg.db")
}

pub fn get_connection(app_handle: &tauri::AppHandle) -> Result<Connection> {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(db_path)?;
    conn.execute_batch("
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
    ").map_err(|e| e)?;
    Ok(conn)
}

pub fn insert_workspace(app_handle: &tauri::AppHandle, workspace: &Workspace) -> Result<()> {
    let conn = get_connection(app_handle)?;
    conn.execute(
        "INSERT INTO workspaces (id, name, path, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            workspace.id,
            workspace.name,
            workspace.path,
            workspace.created_at.as_deref().unwrap_or("") // fallback if none, though schema has default
        ],
    )?;
    Ok(())
}

pub fn get_workspaces(app_handle: &tauri::AppHandle) -> Result<Vec<Workspace>> {
    let conn = get_connection(app_handle)?;
    let mut stmt = conn.prepare("SELECT id, name, path, created_at FROM workspaces ORDER BY created_at DESC")?;
    let workspace_iter = stmt.query_map([], |row| {
        Ok(Workspace {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    let mut workspaces = Vec::new();
    for ws in workspace_iter {
        workspaces.push(ws?);
    }
    Ok(workspaces)
}

pub fn create_workflow_run(app_handle: &tauri::AppHandle, run: &WorkflowRun) -> Result<()> {
    let conn = get_connection(app_handle)?;
    conn.execute(
        "INSERT INTO workflow_runs (id, workspace_id, status, logs_path, start_time, end_time) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            run.id,
            run.workspace_id,
            run.status,
            run.logs_path,
            run.start_time,
            run.end_time
        ],
    )?;
    Ok(())
}

pub fn get_workflow_runs(app_handle: &tauri::AppHandle, workspace_id: &str) -> Result<Vec<WorkflowRun>> {
    let conn = get_connection(app_handle)?;
    let mut stmt = conn.prepare("SELECT id, workspace_id, status, mode, prompt, logs_path, start_time, end_time FROM workflow_runs WHERE workspace_id = ?1 ORDER BY start_time DESC")?;
    let run_iter = stmt.query_map(params![workspace_id], |row| {
        Ok(WorkflowRun {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            status: row.get(2)?,
            mode: row.get(3)?,
            prompt: row.get(4)?,
            logs_path: row.get(5)?,
            start_time: row.get(6)?,
            end_time: row.get(7)?,
        })
    })?;

    let mut runs = Vec::new();
    for run in run_iter {
        runs.push(run?);
    }
    Ok(runs)
}

pub fn update_workflow_run_status(
    app_handle: &tauri::AppHandle, 
    run_id: &str, 
    status: &str, 
    end_time: &str, 
    logs_path: Option<&str>
) -> Result<()> {
    let conn = get_connection(app_handle)?;
    if let Some(path) = logs_path {
        conn.execute(
            "UPDATE workflow_runs SET status = ?1, end_time = ?2, logs_path = ?3 WHERE id = ?4",
            params![status, end_time, path, run_id],
        )?;
    } else {
        conn.execute(
            "UPDATE workflow_runs SET status = ?1, end_time = ?2 WHERE id = ?3",
            params![status, end_time, run_id],
        )?;
    }
    Ok(())
}
