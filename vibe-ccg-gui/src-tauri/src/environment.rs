use chrono::Utc;
use serde::Serialize;
use std::process::{Output, Stdio};
use std::time::Instant;
use tokio::process::Command as AsyncCommand;
use tokio::time::{timeout, Duration};

#[derive(Serialize, Debug, Clone)]
pub struct ToolCheckResult {
    pub id: String,
    pub display_name: String,
    pub candidates: Vec<String>,
    pub status: String,        // "installed" | "not_found" | "error"
    pub resolved_path: Option<String>,
    pub version: Option<String>,
    pub source: Option<String>, // "where" | "which" | "version"
    pub latency_ms: u64,
    pub error: Option<String>,
    pub install_url: Option<String>,
    pub install_hint: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EnvironmentCheckResult {
    pub checked_at: String,
    pub platform: String,
    pub tools: Vec<ToolCheckResult>,
}

#[derive(Copy, Clone)]
struct ToolSpec {
    id: &'static str,
    display_name: &'static str,
    candidates: &'static [&'static str],
    version_args: &'static [&'static str],
    npm_package: Option<&'static str>, // If set, detect via `npm ls -g <pkg>` instead of PATH
    install_url: &'static str,
    install_hint: &'static str,
}

const TOOL_SPECS: &[ToolSpec] = &[
    ToolSpec {
        id: "node",
        display_name: "Node.js",
        candidates: &["node"],
        version_args: &["--version"],
        npm_package: None,
        install_url: "https://nodejs.org/",
        install_hint: "winget install OpenJS.NodeJS.LTS",
    },
    ToolSpec {
        id: "git",
        display_name: "Git",
        candidates: &["git"],
        version_args: &["--version"],
        npm_package: None,
        install_url: "https://git-scm.com/",
        install_hint: "winget install Git.Git",
    },
    ToolSpec {
        id: "claude",
        display_name: "Claude Code CLI",
        candidates: &["claude"],
        version_args: &["--version"],
        npm_package: None,
        install_url: "https://docs.anthropic.com/en/docs/claude-code",
        install_hint: "npm install -g @anthropic-ai/claude-code",
    },
    ToolSpec {
        id: "ccg",
        display_name: "CCG Workflow",
        candidates: &[],
        version_args: &["--version"],
        npm_package: Some("ccg-workflow"),
        install_url: "https://github.com/fengshao1227/ccg-workflow",
        install_hint: "npx ccg-workflow",
    },
    ToolSpec {
        id: "opsx",
        display_name: "OpenSpec CLI",
        candidates: &["openspec", "opsx"],
        version_args: &["--version"],
        npm_package: None,
        install_url: "https://github.com/anthropics/claude-code",
        install_hint: "npm install -g openspec-cli",
    },
    ToolSpec {
        id: "codex",
        display_name: "Codex CLI",
        candidates: &["codex"],
        version_args: &["--version"],
        npm_package: None,
        install_url: "https://github.com/openai/codex",
        install_hint: "npm install -g @openai/codex",
    },
    ToolSpec {
        id: "gemini",
        display_name: "Gemini CLI",
        candidates: &["gemini"],
        version_args: &["--version"],
        npm_package: None,
        install_url: "https://github.com/google-gemini/gemini-cli",
        install_hint: "npm install -g @google/gemini-cli",
    },
];

#[tauri::command]
pub async fn check_environment() -> Result<EnvironmentCheckResult, String> {
    let mut set = tokio::task::JoinSet::new();
    for (index, spec) in TOOL_SPECS.iter().enumerate() {
        let spec = *spec;
        set.spawn(async move { (index, detect_tool(spec).await) });
    }

    let mut results: Vec<(usize, ToolCheckResult)> = Vec::with_capacity(TOOL_SPECS.len());
    while let Some(res) = set.join_next().await {
        match res {
            Ok((index, tool)) => results.push((index, tool)),
            Err(e) => results.push((
                usize::MAX,
                ToolCheckResult {
                    id: "unknown".to_string(),
                    display_name: "Unknown".to_string(),
                    candidates: Vec::new(),
                    status: "error".to_string(),
                    resolved_path: None,
                    version: None,
                    source: None,
                    latency_ms: 0,
                    error: Some(format!("task join failed: {}", e)),
                    install_url: None,
                    install_hint: None,
                },
            )),
        }
    }
    results.sort_by_key(|(index, _)| *index);

    let tools = results.into_iter().map(|(_, tool)| tool).collect();
    Ok(EnvironmentCheckResult {
        checked_at: Utc::now().to_rfc3339(),
        platform: std::env::consts::OS.to_string(),
        tools,
    })
}

async fn detect_tool(spec: ToolSpec) -> ToolCheckResult {
    let start = Instant::now();
    let result = timeout(Duration::from_secs(10), detect_tool_inner(spec)).await;
    let mut tool = match result {
        Ok(tool) => tool,
        Err(_) => ToolCheckResult {
            id: spec.id.to_string(),
            display_name: spec.display_name.to_string(),
            candidates: spec.candidates.iter().map(|c| c.to_string()).collect(),
            status: "error".to_string(),
            resolved_path: None,
            version: None,
            source: None,
            latency_ms: 0,
            error: Some("detection timeout (10s)".to_string()),
            install_url: Some(spec.install_url.to_string()),
            install_hint: Some(spec.install_hint.to_string()),
        },
    };
    tool.latency_ms = start.elapsed().as_millis() as u64;
    tool
}

async fn detect_tool_inner(spec: ToolSpec) -> ToolCheckResult {
    let candidates: Vec<String> = spec.candidates.iter().map(|c| c.to_string()).collect();

    // Special handling for npm packages (e.g. ccg-workflow via npx)
    if let Some(pkg) = spec.npm_package {
        return detect_npm_package(spec, pkg).await;
    }

    let (resolved_path, source) = resolve_path(spec).await;

    let mut status = "not_found".to_string();
    let mut version: Option<String> = None;
    let mut error: Option<String> = None;
    let mut used_source = source.clone();

    // 1. If `where` found a full path, try it first (most reliable)
    if let Some(path) = resolved_path.as_deref() {
        let paths_to_try: Vec<String> = if cfg!(windows) {
            let mut v = Vec::new();
            // On Windows, prefer .cmd variant if the path has no extension
            if !path.ends_with(".cmd") && !path.ends_with(".exe") {
                v.push(format!("{}.cmd", path));
            }
            v.push(path.to_string());
            v
        } else {
            vec![path.to_string()]
        };

        for p in &paths_to_try {
            match run_version(p, spec.version_args).await {
                Ok(ver) => {
                    version = Some(ver);
                    status = "installed".to_string();
                    used_source = Some("where".to_string());
                    break;
                }
                Err(e) => {
                    error = Some(e);
                }
            }
        }
    }

    // 2. Fall back to bare candidate names (relies on PATH)
    if version.is_none() {
        for &candidate in spec.candidates {
            match run_version(candidate, spec.version_args).await {
                Ok(ver) => {
                    version = Some(ver);
                    status = "installed".to_string();
                    if used_source.is_none() {
                        used_source = Some("version".to_string());
                    }
                    break;
                }
                Err(e) => {
                    error = Some(e);
                }
            }
        }
    }

    if status == "not_found" && error.is_none() {
        error = Some("not found in PATH".to_string());
    }

    ToolCheckResult {
        id: spec.id.to_string(),
        display_name: spec.display_name.to_string(),
        candidates,
        status,
        resolved_path,
        version,
        source: used_source,
        latency_ms: 0,
        error,
        install_url: Some(spec.install_url.to_string()),
        install_hint: Some(spec.install_hint.to_string()),
    }
}

/// Detect an npm package by checking global install, then scanning npx cache.
/// Never triggers network downloads — pure local detection.
async fn detect_npm_package(spec: ToolSpec, pkg: &str) -> ToolCheckResult {
    let candidates: Vec<String> = if spec.candidates.is_empty() {
        vec![pkg.to_string()]
    } else {
        spec.candidates.iter().map(|c| c.to_string()).collect()
    };

    let mut status = "not_found".to_string();
    let mut version: Option<String> = None;
    let mut resolved_path: Option<String> = None;
    let mut error: Option<String> = None;
    let mut source: Option<String> = None;

    // 1. Check global install: npm ls -g <pkg> --depth=0
    let check_cmd = format!("npm ls -g {} --depth=0", pkg);
    if let Ok(output) = run_cmd_output(&check_cmd).await {
        let stdout = decode_output(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.contains(pkg) {
                if let Some(at_pos) = trimmed.rfind('@') {
                    let ver = trimmed[at_pos + 1..].trim();
                    if !ver.is_empty() {
                        version = Some(ver.to_string());
                        status = "installed".to_string();
                        source = Some("npm-global".to_string());
                    }
                }
                break;
            }
        }
        // Resolve global path
        if status == "installed" {
            if let Ok(root_output) = run_cmd_output("npm root -g").await {
                if root_output.status.success() {
                    let root_stdout = decode_output(&root_output.stdout);
                    if let Some(root) = first_non_empty_line(&root_stdout) {
                        resolved_path = Some(format!("{}\\{}", root, pkg));
                    }
                }
            }
        }
    }

    // 2. If not global, scan npx cache directory (deterministic, no network)
    if version.is_none() {
        if let Some((ver, path)) = scan_npx_cache(pkg).await {
            version = Some(ver);
            resolved_path = Some(path);
            status = "installed".to_string();
            source = Some("npx-cache".to_string());
        }
    }

    if status == "not_found" {
        error = Some("not found in global npm or npx cache".to_string());
    }

    ToolCheckResult {
        id: spec.id.to_string(),
        display_name: spec.display_name.to_string(),
        candidates,
        status,
        resolved_path,
        version,
        source,
        latency_ms: 0,
        error,
        install_url: Some(spec.install_url.to_string()),
        install_hint: Some(spec.install_hint.to_string()),
    }
}

/// Scan npx cache for a cached package. Uses `npm config get cache` to locate
/// the cache dir, then checks _npx/*/node_modules/<pkg>/package.json.
/// Returns (version, path) if found. Pure filesystem read, zero side effects.
async fn scan_npx_cache(pkg: &str) -> Option<(String, String)> {
    // Get the actual npm cache path (e.g. C:\Users\xx\AppData\Local\npm-cache)
    let cache_root = get_npm_cache_dir().await?;
    let npx_cache = std::path::PathBuf::from(&cache_root).join("_npx");

    let mut entries = tokio::fs::read_dir(&npx_cache).await.ok()?;
    let mut best: Option<(String, String)> = None;

    while let Ok(Some(entry)) = entries.next_entry().await {
        let pkg_json_path = entry.path()
            .join("node_modules")
            .join(pkg)
            .join("package.json");

        if let Ok(content) = tokio::fs::read_to_string(&pkg_json_path).await {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ver) = json.get("version").and_then(|v| v.as_str()) {
                    let pkg_path = entry.path().join("node_modules").join(pkg);
                    let ver_str = ver.to_string();
                    // Keep the highest version if multiple caches exist
                    if best.as_ref().map_or(true, |(v, _)| ver_str > *v) {
                        best = Some((ver_str, pkg_path.to_string_lossy().to_string()));
                    }
                }
            }
        }
    }
    best
}

/// Get npm cache directory via `npm config get cache`.
async fn get_npm_cache_dir() -> Option<String> {
    if let Ok(output) = run_cmd_output("npm config get cache").await {
        if output.status.success() {
            return first_non_empty_line(&decode_output(&output.stdout));
        }
    }
    // Fallback: common default paths
    if let Ok(home) = std::env::var("LOCALAPPDATA") {
        let default = std::path::PathBuf::from(&home).join("npm-cache");
        if default.exists() {
            return Some(default.to_string_lossy().to_string());
        }
    }
    if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        let default = std::path::PathBuf::from(&home).join(".npm");
        if default.exists() {
            return Some(default.to_string_lossy().to_string());
        }
    }
    None
}

async fn resolve_path(spec: ToolSpec) -> (Option<String>, Option<String>) {
    for &candidate in spec.candidates {
        if let Ok(output) = run_cmd_output(&format!("where {}", candidate)).await {
            if output.status.success() {
                let stdout = decode_output(&output.stdout);
                // where returns multiple lines; pick the first .cmd or .exe, else first line
                let lines: Vec<&str> = stdout.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();
                // Prefer .cmd or .exe variant
                let best = lines.iter()
                    .find(|l| l.ends_with(".cmd") || l.ends_with(".exe"))
                    .or(lines.first());
                if let Some(line) = best {
                    return (Some(line.to_string()), Some("where".to_string()));
                }
            }
        }
    }
    (None, None)
}

async fn run_version(cmd: &str, args: &[&str]) -> Result<String, String> {
    let full_cmd = format!("{} {}", cmd, args.join(" "));
    let output = run_cmd_output(&full_cmd).await?;
    if !output.status.success() {
        let stderr = decode_output(&output.stderr);
        let message = first_non_empty_line(&stderr)
            .unwrap_or_else(|| "version command failed".to_string());
        return Err(message);
    }
    parse_version_output(&output).ok_or_else(|| "version output empty".to_string())
}

/// Runs a command string via `cmd /C chcp 65001 >nul && <command>` on Windows,
/// or `sh -c <command>` on other platforms.
/// Using chcp 65001 forces UTF-8 output so we don't get GBK garbled text.
async fn run_cmd_output(command: &str) -> Result<Output, String> {
    if cfg!(windows) {
        AsyncCommand::new("cmd")
            .args(["/C", &format!("chcp 65001 >nul 2>nul & {}", command)])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| e.to_string())
    } else {
        AsyncCommand::new("sh")
            .args(["-c", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| e.to_string())
    }
}

/// Decode command output bytes. Try UTF-8 first, fall back to lossy.
fn decode_output(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|_| String::from_utf8_lossy(bytes).to_string())
}

fn parse_version_output(output: &Output) -> Option<String> {
    let stdout = decode_output(&output.stdout);
    if let Some(line) = first_non_empty_line(&stdout) {
        return Some(line);
    }
    let stderr = decode_output(&output.stderr);
    first_non_empty_line(&stderr)
}

fn first_non_empty_line(text: &str) -> Option<String> {
    text.lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
        .map(|line| line.to_string())
}
