use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use crate::{Workspace, WorkflowRun};

// Helper function to get the database path
pub fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_dir = app_handle.path().app_data_dir().expect("Failed to get app data dir");
    // tauri-plugin-sql stores the db in the app local data dir
    app_dir.join("vibe-ccg.db")
}

pub fn get_connection(app_handle: &tauri::AppHandle) -> Result<Connection> {
    let db_path = get_db_path(app_handle);
    Connection::open(db_path)
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
    let mut stmt = conn.prepare("SELECT id, workspace_id, status, logs_path, start_time, end_time FROM workflow_runs WHERE workspace_id = ?1 ORDER BY start_time DESC")?;
    let run_iter = stmt.query_map(params![workspace_id], |row| {
        Ok(WorkflowRun {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            status: row.get(2)?,
            logs_path: row.get(3)?,
            start_time: row.get(4)?,
            end_time: row.get(5)?,
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
