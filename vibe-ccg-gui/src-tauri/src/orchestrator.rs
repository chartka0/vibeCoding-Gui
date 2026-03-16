use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command as AsyncCommand;
use uuid::Uuid;
use chrono::Utc;
use tauri::{AppHandle, Emitter};

/// Find git-bash (bash.exe) path for injecting CLAUDE_CODE_GIT_BASH_PATH.
/// Claude Code checks this env var to verify git-bash is available.
/// Returns the bash.exe path as a Windows-native path string.
fn find_git_bash_path() -> Option<String> {
    // 1. Already set by user — trust it
    if let Ok(path) = std::env::var("CLAUDE_CODE_GIT_BASH_PATH") {
        if PathBuf::from(&path).exists() {
            return Some(path);
        }
    }

    // 2. Common Git for Windows install paths
    let candidates = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
        r"D:\RuanJian\Git\bin\bash.exe",
        r"C:\Git\bin\bash.exe",
    ];
    for path in &candidates {
        if PathBuf::from(path).exists() {
            return Some(path.to_string());
        }
    }

    // 3. Derive from `where git` output
    if let Ok(output) = std::process::Command::new("where").arg("git").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let git_path = PathBuf::from(line.trim());
                if let Some(parent) = git_path.parent().and_then(|p| p.parent()) {
                    let bash = parent.join("bin").join("bash.exe");
                    if bash.exists() {
                        return Some(bash.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Build a claude command with CLAUDE_CODE_GIT_BASH_PATH injected.
/// On Windows, Claude Code requires this env var to pass its git-bash check.
/// We run claude directly via cmd /C so the PATH is correct,
/// but set CLAUDE_CODE_GIT_BASH_PATH so Claude's internal check passes.
fn build_claude_cmd() -> AsyncCommand {
    let mut cmd = if cfg!(windows) {
        let mut c = AsyncCommand::new("cmd");
        c.arg("/C").arg("claude");
        // Inject git-bash path so Claude Code's internal check passes
        if let Some(bash_path) = find_git_bash_path() {
            c.env("CLAUDE_CODE_GIT_BASH_PATH", bash_path);
        }
        c
    } else {
        AsyncCommand::new("claude")
    };
    cmd
}

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

    // 2. Spawn claude -p subprocess (with CLAUDE_CODE_GIT_BASH_PATH on Windows)
    let mut output_accumulator = String::new();

    let mut cmd = build_claude_cmd();
    cmd.args(["-p", "--output-format", "stream-json", "--verbose", "--dangerously-skip-permissions"]);
    if let Some(sid) = session_id.clone() {
        cmd.args(["--session-id", &sid]);
    }

    cmd.current_dir(&workspace_path)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            update_step_status(&app, &step_id, "failed", None, Some(&e.to_string()))?;
            return Err(e.to_string());
        }
    };

    // Write prompt via stdin then close
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(prompt.as_bytes()).await;
        drop(stdin);
    }

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

    let mut cmd = build_claude_cmd();
    cmd.args([
        "--output-format", "stream-json",
        "--verbose",
        "--dangerously-skip-permissions",
    ]);

    cmd.current_dir(&workspace_path)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;

    // Pipe the prompt (including /skill prefix) via stdin
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(full_prompt.as_bytes()).await;
        drop(stdin);
    }

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

// ───────────────────────────────────────────────────
// Per-step workflow execution with session continuity
// ───────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct StepDonePayload {
    pub step_index: i32,
    pub status: String,
    pub session_id: Option<String>,
}

/// Run a single workflow step via `claude -p` with session continuity.
/// - First step: creates a new claude session via `--session-id`.
/// - Subsequent steps: resumes the session so Claude retains full context.
/// - Parses stream-json output to emit clean text for the frontend.
/// - Stores step output in DB. Retry = call again with same params.
#[tauri::command]
pub async fn run_workflow_step(
    app: AppHandle,
    run_id: String,
    step_index: i32,
    step_name: String,
    workspace_path: String,
    prompt: String,
    session_id: String,
) -> Result<String, String> {
    let step_id = Uuid::new_v4().to_string();
    let start_time = Utc::now().to_rfc3339();
    let log_event = format!("wf-step-log-{}-{}", run_id, step_index);
    let done_event = format!("wf-step-done-{}", run_id);

    // Upsert: delete old step record if retrying
    {
        let conn = crate::db::get_connection(&app).map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM workflow_steps WHERE run_id = ?1 AND step_index = ?2",
            rusqlite::params![&run_id, &step_index],
        ).ok();
        conn.execute(
            "INSERT INTO workflow_steps (id, run_id, step_index, step_name, status, input_prompt, session_id, start_time)
             VALUES (?1, ?2, ?3, ?4, 'running', ?5, ?6, ?7)",
            rusqlite::params![&step_id, &run_id, &step_index, &step_name, &prompt, &session_id, &start_time],
        ).map_err(|e| e.to_string())?;
    }

    // Build claude command with git-bash path injection
    let mut cmd = build_claude_cmd();
    cmd.args([
        "--session-id", &session_id,
        "--output-format", "stream-json",
        "--verbose",
        "--dangerously-skip-permissions",
    ]);

    cmd.current_dir(&workspace_path)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        let _ = update_step_status(&app, &step_id, "failed", None, Some(&e.to_string()));
        e.to_string()
    })?;

    // Write prompt via stdin then close — claude reads piped stdin as the prompt
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(prompt.as_bytes()).await;
        drop(stdin);
    }

    // Stream & parse stdout
    let app_out = app.clone();
    let log_ev = log_event.clone();
    let stdout_handle = if let Some(stdout) = child.stdout.take() {
        Some(tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            let mut saw_deltas = false;
            let mut streamed_text = String::new();
            let mut result_text: Option<String> = None;
            let mut detected_sid: Option<String> = None;

            while let Ok(Some(line)) = reader.next_line().await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    // Grab session_id from any event
                    if detected_sid.is_none() {
                        if let Some(sid) = json.get("session_id").and_then(|v| v.as_str()) {
                            detected_sid = Some(sid.to_string());
                        }
                    }

                    let evt_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    match evt_type {
                        "content_block_delta" => {
                            if let Some(text) = json.get("delta")
                                .and_then(|d| d.get("text"))
                                .and_then(|v| v.as_str())
                            {
                                saw_deltas = true;
                                streamed_text.push_str(text);
                                let _ = app_out.emit(&log_ev, text);
                            }
                        }
                        "result" => {
                            result_text = json.get("result")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            // Only emit if we didn't stream deltas (avoid duplicate)
                            if !saw_deltas {
                                if let Some(ref t) = result_text {
                                    streamed_text = t.clone();
                                    let _ = app_out.emit(&log_ev, t.as_str());
                                }
                            }
                        }
                        "assistant" => {
                            // Some CLI versions emit this format
                            if let Some(text) = json.get("result")
                                .or_else(|| json.get("text"))
                                .and_then(|v| v.as_str())
                            {
                                if !saw_deltas && !text.is_empty() {
                                    streamed_text.push_str(text);
                                    let _ = app_out.emit(&log_ev, text);
                                }
                            }
                        }
                        _ => {} // skip metadata: init, message_start, message_stop, etc.
                    }
                } else if !line.trim().is_empty() {
                    // Non-JSON line
                    streamed_text.push_str(&line);
                    streamed_text.push('\n');
                    let _ = app_out.emit(&log_ev, line.as_str());
                }
            }

            let final_text = result_text.unwrap_or(streamed_text);
            (detected_sid, final_text)
        }))
    } else {
        None
    };

    // Stream stderr
    let app_err = app.clone();
    let log_ev2 = log_event.clone();
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = app_err.emit(&log_ev2, format!("[stderr] {}", line));
            }
        });
    }

    // Wait for exit
    let exit = child.wait().await.map_err(|e| e.to_string())?;
    let (detected_sid, full_text) = match stdout_handle {
        Some(h) => h.await.map_err(|e| e.to_string())?,
        None => (None, String::new()),
    };

    let final_status = if exit.success() { "success" } else { "failed" };
    let error_msg = if exit.success() { None } else { Some("claude process exited with error") };

    // Update DB
    {
        let conn = crate::db::get_connection(&app).map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE workflow_steps SET status=?1, output_text=?2, session_id=?3, end_time=?4, error_message=?5 WHERE id=?6",
            rusqlite::params![
                final_status,
                &full_text,
                detected_sid.as_deref().unwrap_or(&session_id),
                Utc::now().to_rfc3339(),
                error_msg,
                &step_id
            ],
        ).map_err(|e| e.to_string())?;
    }

    // Emit completion event
    let _ = app.emit(&done_event, StepDonePayload {
        step_index,
        status: final_status.to_string(),
        session_id: detected_sid.or(Some(session_id)),
    });

    Ok(step_id)
}
