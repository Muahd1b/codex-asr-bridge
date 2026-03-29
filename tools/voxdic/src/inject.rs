use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::config::{InjectApp, RewriteMode};
use crate::transform;
use crate::util::truncate;

#[derive(Debug, Clone)]
pub struct InjectResult {
    pub front_app: String,
    pub chunks: usize,
}

#[derive(Debug, Clone)]
pub struct SelectionRewriteResult {
    pub front_app: String,
    pub before_chars: usize,
    pub after_chars: usize,
}

pub fn inject_focused_text(
    text: &str,
    app_mode: InjectApp,
    chunk_chars: usize,
) -> Result<InjectResult> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Empty transcript, nothing to inject"));
    }

    let front = frontmost_process_name()?;
    if matches!(app_mode, InjectApp::TerminalOnly) && !is_terminal_app(&front) {
        return Err(anyhow!(
            "Focused app is '{}', not Terminal/iTerm. Bring your CLI tab to front or switch mode to any_focused.",
            front
        ));
    }

    let chunks = split_for_injection(trimmed, chunk_chars.max(40));
    for chunk in &chunks {
        inject_chunk(chunk)?;
    }

    Ok(InjectResult {
        front_app: front,
        chunks: chunks.len(),
    })
}

pub fn frontmost_app_name() -> Result<String> {
    frontmost_process_name()
}

pub fn inject_text_to_target_app(
    text: &str,
    app_mode: InjectApp,
    chunk_chars: usize,
    target_app: &str,
) -> Result<InjectResult> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Empty transcript, nothing to inject"));
    }

    if matches!(app_mode, InjectApp::TerminalOnly) && !is_terminal_app(target_app) {
        return Err(anyhow!(
            "Target app is '{}', not Terminal/iTerm. Set inject mode to auto or any_focused.",
            target_app
        ));
    }

    activate_app(target_app)?;

    let chunks = split_for_injection(trimmed, chunk_chars.max(40));
    for chunk in &chunks {
        inject_chunk(chunk)?;
    }

    Ok(InjectResult {
        front_app: target_app.to_string(),
        chunks: chunks.len(),
    })
}

pub fn rewrite_selected_text(
    app_mode: InjectApp,
    rewrite_mode: RewriteMode,
) -> Result<SelectionRewriteResult> {
    if matches!(rewrite_mode, RewriteMode::None) {
        return Err(anyhow!(
            "Rewrite mode is none. Press 'p' to set a rewrite mode first."
        ));
    }

    let front = frontmost_process_name()?;
    if matches!(app_mode, InjectApp::TerminalOnly) && !is_terminal_app(&front) {
        return Err(anyhow!(
            "Focused app is '{}', not Terminal/iTerm. Bring your CLI tab to front or switch mode to any_focused.",
            front
        ));
    }

    let original_clipboard = clipboard_read().unwrap_or_default();
    let sentinel = "__VOXDIC_SENTINEL__";
    clipboard_write(sentinel)?;

    if let Err(err) = send_command_shortcut("c") {
        let _ = clipboard_write(&original_clipboard);
        return Err(err);
    }
    thread::sleep(Duration::from_millis(140));

    let selected = clipboard_read()?;
    if selected == sentinel {
        let _ = clipboard_write(&original_clipboard);
        return Err(anyhow!(
            "No selected text copied from focused app. Select text first, then press 'c'."
        ));
    }

    let rewritten = transform::apply_rewrite_mode(&selected, rewrite_mode);
    if rewritten.trim().is_empty() {
        let _ = clipboard_write(&original_clipboard);
        return Err(anyhow!("Rewritten text is empty, aborting replacement."));
    }

    clipboard_write(&rewritten)?;
    if let Err(err) = send_command_shortcut("v") {
        let _ = clipboard_write(&original_clipboard);
        return Err(err);
    }

    thread::sleep(Duration::from_millis(120));
    let _ = clipboard_write(&original_clipboard);

    Ok(SelectionRewriteResult {
        front_app: front,
        before_chars: selected.chars().count(),
        after_chars: rewritten.chars().count(),
    })
}

fn frontmost_process_name() -> Result<String> {
    let lines = vec![
        "tell application \"System Events\"",
        "set p to first process whose frontmost is true",
        "return name of p",
        "end tell",
    ];
    let out = run_osascript(&lines)?;
    if out.is_empty() {
        return Err(anyhow!("Could not detect focused app"));
    }
    Ok(out)
}

fn send_command_shortcut(key: &str) -> Result<()> {
    let escaped = escape_applescript(key);
    let script = vec![
        "tell application \"System Events\"".to_string(),
        format!("keystroke \"{}\" using {{command down}}", escaped),
        "end tell".to_string(),
    ];
    let _ = run_osascript_owned(script)?;
    Ok(())
}

fn activate_app(app_name: &str) -> Result<()> {
    let escaped = escape_applescript(app_name);
    let script = vec![
        "tell application \"System Events\"".to_string(),
        format!(
            "if not (exists process \"{}\") then error \"Target app not running\" number -1728",
            escaped
        ),
        format!("set frontmost of process \"{}\" to true", escaped),
        "end tell".to_string(),
    ];
    let _ = run_osascript_owned(script)?;
    Ok(())
}

fn clipboard_read() -> Result<String> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| anyhow!("Failed running pbpaste: {}", e))?;
    if !output.status.success() {
        return Err(anyhow!(
            "pbpaste failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn clipboard_write(text: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed running pbcopy: {}", e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| anyhow!("Failed writing to pbcopy stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| anyhow!("Failed waiting for pbcopy: {}", e))?;
    if !output.status.success() {
        return Err(anyhow!(
            "pbcopy failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

fn inject_chunk(chunk: &str) -> Result<()> {
    // Preserve newlines manually by sending Enter between line chunks.
    let lines = chunk.split('\n').collect::<Vec<_>>();
    for (idx, line) in lines.iter().enumerate() {
        if !line.is_empty() {
            let escaped = escape_applescript(line);
            let script = vec![
                "tell application \"System Events\"".to_string(),
                format!("keystroke \"{}\"", escaped),
                "end tell".to_string(),
            ];
            let _ = run_osascript_owned(script)?;
        }
        if idx + 1 < lines.len() {
            let script = vec![
                "tell application \"System Events\"".to_string(),
                "key code 36".to_string(), // Enter
                "end tell".to_string(),
            ];
            let _ = run_osascript_owned(script)?;
        }
    }
    Ok(())
}

fn split_for_injection(text: &str, max_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();

    for token in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(token);
            continue;
        }

        let maybe_len = current.chars().count() + 1 + token.chars().count();
        if maybe_len > max_chars {
            out.push(current);
            current = token.to_string();
        } else {
            current.push(' ');
            current.push_str(token);
        }
    }

    if !current.is_empty() {
        out.push(current);
    }

    if out.is_empty() {
        vec![text.to_string()]
    } else {
        out
    }
}

fn is_terminal_app(name: &str) -> bool {
    matches!(name, "Terminal" | "iTerm2" | "iTerm" | "Warp")
}

fn run_osascript(lines: &[&str]) -> Result<String> {
    let owned = lines.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    run_osascript_owned(owned)
}

fn run_osascript_owned(lines: Vec<String>) -> Result<String> {
    let output = Command::new("osascript")
        .args(
            lines
                .iter()
                .flat_map(|line| ["-e".to_string(), line.clone()]),
        )
        .output()
        .map_err(|e| anyhow!("Failed running osascript: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        return Ok(stdout);
    }

    let msg = if !stderr.is_empty() { stderr } else { stdout };
    if msg.contains("not allowed to send keystrokes") || msg.contains("(1002)") {
        return Err(anyhow!(
            "macOS denied keystroke injection (TCC 1002). Enable Accessibility and Input Monitoring for your terminal app, then restart Voxdic."
        ));
    }
    Err(anyhow!("AppleScript error: {}", truncate(&msg, 220)))
}

fn escape_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_chunks_respects_limit() {
        let parts = split_for_injection("one two three four five", 8);
        assert!(parts.len() >= 2);
        assert!(parts.iter().all(|p| p.chars().count() <= 8));
    }

    #[test]
    fn escapes_for_applescript() {
        let escaped = escape_applescript("a\"b\\c");
        assert_eq!(escaped, "a\\\"b\\\\c");
    }
}
