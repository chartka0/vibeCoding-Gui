use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use uuid::Uuid;
use chrono::Utc;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowStep {
    pub id: String,
    pub run_id: String,
    pub step_index: i32,
    pub step_name: String,
    pub status: String,
    pub input_prompt: Option<String>,
    pub output_text: Option<String>,
    pub output_summary: Option<String>,
    pub session_id: Option<String>,
    pub codex_session: Option<String>,
    pub gemini_session: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct StepEventPayload {
    pub step_index: i32,
    pub status: String,
    pub message: String,
}

#[tauri::command]
pub async fn start_workflow_run(
    app: AppHandle,
    workspace_id: String,
    mode: String,
    prompt: String,
) -> Result<String, String> {
    let run_id = Uuid::new_v4().to_string();
    
    let run_sql = "INSERT INTO workflow_runs (id, workspace_id, mode, prompt, status, start_time) VALUES (?1, ?2, ?3, ?4, 'Running', ?5)";
    
    let conn = crate::db::get_connection(&app).map_err(|e| e.to_string())?;
    conn.execute(
        run_sql,
        rusqlite::params![&run_id, &workspace_id, &mode, &prompt, Utc::now().to_rfc3339()],
    )
    .map_err(|e| e.to_string())?;

    Ok(run_id)
}

#[tauri::command]
pub async fn execute_step(
    app: AppHandle,
    run_id: String,
    step_index: i32,
    step_name: String,
    workspace_path: String,
    prompt: String,
    session_id: Option<String>,
) -> Result<WorkflowStep, String> {
    let step_id = Uuid::new_v4().to_string();
    let start_time = Utc::now().to_rfc3339();

    // 1. Create step record in SQLite
    let insert_sql = "
        INSERT INTO workflow_steps 
        (id, run_id, step_index, step_name, status, input_prompt, start_time, session_id) 
        VALUES (?1, ?2, ?3, ?4, 'running', ?5, ?6, ?7)
    ";
    
    {
        let conn = crate::db::get_connection(&app).map_err(|e| e.to_string())?;
        conn.execute(
            insert_sql,
            rusqlite::params![
                &step_id, &run_id, &step_index, &step_name, &prompt, &start_time, &session_id
            ],
        ).map_err(|e| format!("DB Insert failed: {}", e))?;
    }

    let emit_event = format!("step-update-{}", run_id);
    let _ = app.emit(&emit_event, StepEventPayload {
        step_index,
        status: "running".to_string(),
        message: "Step started".to_string(),
    });

    // 2. Spawn claude -p subprocess
    let mut cmd = AsyncCommand::new(if cfg!(windows) { "cmd" } else { "sh" });
    
    // Fallback Mock execution for development if claude is missing
    let mut output_accumulator = String::new();
    let is_mock = false; // Set to false in production to run actual claude CLI
    
    if is_mock {
        cmd.arg(if cfg!(windows) { "/C" } else { "-c" });
        cmd.arg(format!("echo [JSON_START] && echo {{\"status\": \"success\", \"step\": {}, \"message\": \"Mock result for {}: {}\"}} && timeout 2 > NUL", step_index, step_name, prompt));
    } else {
        // Real claude CLI via cmd /C on Windows
        cmd.arg("/C");
        let mut claude_cmd = format!("claude -p \"{}\" --output-format stream-json --dangerously-skip-permissions", prompt.replace('"', "'"));
        if let Some(sid) = session_id.clone() {
            claude_cmd.push_str(&format!(" --session-id {}", sid));
        }
        cmd.arg(claude_cmd);
    }

    cmd.current_dir(&workspace_path)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            update_step_status(&app, &step_id, "failed", None, Some(&e.to_string()))?;
            return Err(e.to_string());
        }
    };

    // 3. Stream output
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app.emit(&format!("step-log-{}", run_id), format!("[Step {}] {}", step_index, line));
            output_accumulator.push_str(&line);
            output_accumulator.push('\n');
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let final_status = if status.success() { "success" } else { "failed" };

    // 4. Update SQLite
    update_step_status(&app, &step_id, final_status, Some(&output_accumulator), None)?;

    let _ = app.emit(&emit_event, StepEventPayload {
        step_index,
        status: final_status.to_string(),
        message: "Step completed".to_string(),
    });

    // Fetch and return the updated step
    get_step_by_id(&app, &step_id)
}

fn update_step_status(
    app: &AppHandle, 
    step_id: &str, 
    status: &str, 
    output: Option<&str>, 
    error: Option<&str>
) -> Result<(), String> {
    let sql = "UPDATE workflow_steps SET status = ?1, output_text = ?2, error_message = ?3, end_time = ?4 WHERE id = ?5";
    let conn = crate::db::get_connection(app).map_err(|e| e.to_string())?;
    conn.execute(
        sql,
        rusqlite::params![status, output, error, Utc::now().to_rfc3339(), step_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

fn get_step_by_id(app: &AppHandle, step_id: &str) -> Result<WorkflowStep, String> {
    let conn = crate::db::get_connection(app).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, run_id, step_index, step_name, status, input_prompt, output_text, output_summary, session_id, codex_session, gemini_session, start_time, end_time, error_message FROM workflow_steps WHERE id = ?1").map_err(|e| e.to_string())?;
    
    let step = stmt.query_row(rusqlite::params![step_id], |row| -> rusqlite::Result<WorkflowStep> {
        Ok(WorkflowStep {
            id: row.get(0)?,
            run_id: row.get(1)?,
            step_index: row.get(2)?,
            step_name: row.get(3)?,
            status: row.get(4)?,
            input_prompt: row.get(5)?,
            output_text: row.get(6)?,
            output_summary: row.get(7)?,
            session_id: row.get(8)?,
            codex_session: row.get(9)?,
            gemini_session: row.get(10)?,
            start_time: row.get(11)?,
            end_time: row.get(12)?,
            error_message: row.get(13)?,
        })
    }).map_err(|e| e.to_string())?;
    
    Ok(step)
}

#[tauri::command]
pub async fn get_run_steps(app: AppHandle, run_id: String) -> Result<Vec<WorkflowStep>, String> {
    let conn = crate::db::get_connection(&app).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, run_id, step_index, step_name, status, input_prompt, output_text, output_summary, session_id, codex_session, gemini_session, start_time, end_time, error_message FROM workflow_steps WHERE run_id = ?1 ORDER BY step_index ASC").map_err(|e| e.to_string())?;
    
    let steps = stmt.query_map(rusqlite::params![run_id], |row| -> rusqlite::Result<WorkflowStep> {
        Ok(WorkflowStep {
            id: row.get(0)?,
            run_id: row.get(1)?,
            step_index: row.get(2)?,
            step_name: row.get(3)?,
            status: row.get(4)?,
            input_prompt: row.get(5)?,
            output_text: row.get(6)?,
            output_summary: row.get(7)?,
            session_id: row.get(8)?,
            codex_session: row.get(9)?,
            gemini_session: row.get(10)?,
            start_time: row.get(11)?,
            end_time: row.get(12)?,
            error_message: row.get(13)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
    
    Ok(steps)
}

#[tauri::command]
pub async fn retry_step(app: AppHandle, step_id: String) -> Result<String, String> {
    update_step_status(&app, &step_id, "pending", None, None)?;
    Ok("Step reset".to_string())
}

/// Run a CCG slash command (e.g. /ccg:workflow, /ccg:frontend) by spawning
/// `claude "/ccg:xxx {prompt}"` in the workspace directory and streaming
/// stdout/stderr back to the frontend via Tauri events.
#[tauri::command]
pub async fn run_ccg_command(
    app: AppHandle,
    run_id: String,
    workspace_path: String,
    skill: String,   // e.g. "ccg:workflow", "ccg:frontend", "ccg:backend"
    prompt: String,
) -> Result<(), String> {
    let full_prompt = format!("/{} {}", skill, prompt);
    let log_event = format!("step-log-{}", run_id);

    let bash_path = std::env::var("CLAUDE_CODE_GIT_BASH_PATH")
        .unwrap_or_else(|_| "D:/RuanJian/Git/usr/bin/bash.exe".to_string());
    let mut cmd = AsyncCommand::new(&bash_path);
    let bash_cmd = format!(
        "CLAUDE_CODE_GIT_BASH_PATH='{}' claude -p '{}' --output-format stream-json --dangerously-skip-permissions",
        bash_path.replace('\\', "/"),
        full_prompt.replace('\'', "'\\''")
    );
    cmd.args(["-c", &bash_cmd])
       .current_dir(&workspace_path)
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;

    // Stream stdout
    let app_clone = app.clone();
    let log_event_clone = log_event.clone();
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout).lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app_clone.emit(&log_event_clone, line);
            }
        });
    }

    // Stream stderr
    let app_clone2 = app.clone();
    let log_event_clone2 = log_event.clone();
    if let Some(stderr) = child.stderr.take() {
        let mut reader = BufReader::new(stderr).lines();
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app_clone2.emit(&log_event_clone2, format!("[stderr] {}", line));
            }
        });
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let final_status = if status.success() { "done" } else { "error" };
    let _ = app.emit(&format!("run-done-{}", run_id), final_status);

    Ok(())
}
