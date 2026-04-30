//! Output renderers for `cvg task ...` commands.
//!
//! Three modes per the CLI's global `--output` flag:
//!
//! - `human` (default): a compact one-line-per-task table for lists,
//!   a multi-line summary for a single task. Uses no color (CLI must
//!   stay screen-reader friendly per CONSTITUTION § P3).
//! - `json`: pretty-printed JSON, the same shape the daemon returns.
//! - `plain`: bare task ids only — designed for shell pipelines like
//!   `id=$(cvg task create ... --output plain)`.

use super::OutputMode;
use anyhow::Result;
use serde_json::Value;

/// Render a single task in the chosen output mode.
pub(crate) fn render_task(task: &Value, output: OutputMode) -> Result<()> {
    match output {
        OutputMode::Human => render_task_human(task),
        OutputMode::Json => {
            println!("{}", serde_json::to_string_pretty(task)?);
            Ok(())
        }
        OutputMode::Plain => {
            if let Some(id) = task.get("id").and_then(Value::as_str) {
                println!("{id}");
            }
            Ok(())
        }
    }
}

/// Render a list of tasks in the chosen output mode.
pub(crate) fn render_task_list(tasks: &Value, output: OutputMode) -> Result<()> {
    match output {
        OutputMode::Human => render_task_list_human(tasks),
        OutputMode::Json => {
            println!("{}", serde_json::to_string_pretty(tasks)?);
            Ok(())
        }
        OutputMode::Plain => {
            if let Some(arr) = tasks.as_array() {
                for t in arr {
                    if let Some(id) = t.get("id").and_then(Value::as_str) {
                        println!("{id}");
                    }
                }
            }
            Ok(())
        }
    }
}

fn render_task_human(task: &Value) -> Result<()> {
    let id = task.get("id").and_then(Value::as_str).unwrap_or("?");
    let status = task.get("status").and_then(Value::as_str).unwrap_or("?");
    let wave = task.get("wave").and_then(Value::as_i64).unwrap_or(0);
    let sequence = task.get("sequence").and_then(Value::as_i64).unwrap_or(0);
    let title = task.get("title").and_then(Value::as_str).unwrap_or("");
    let agent = task
        .get("agent_id")
        .and_then(Value::as_str)
        .unwrap_or("(none)");
    let description = task.get("description").and_then(Value::as_str);
    let evidence = task
        .get("evidence_required")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();

    println!("Task {id}");
    println!("  status:   {status}");
    println!("  wave:     {wave}.{sequence:02}");
    println!("  title:    {title}");
    println!("  agent:    {agent}");
    if !evidence.is_empty() {
        println!("  evidence: {evidence}");
    }
    if let Some(d) = description {
        if !d.is_empty() {
            println!("  description:");
            for line in d.lines() {
                println!("    {line}");
            }
        }
    }
    Ok(())
}

fn render_task_list_human(tasks: &Value) -> Result<()> {
    let arr = match tasks.as_array() {
        Some(a) => a,
        None => return Ok(()),
    };
    if arr.is_empty() {
        println!("(no tasks)");
        return Ok(());
    }
    println!("{:<5} {:<11} {:<8} TITLE", "WAVE", "STATUS", "ID");
    for t in arr {
        let id = t
            .get("id")
            .and_then(Value::as_str)
            .map(|s| &s[..s.len().min(8)])
            .unwrap_or("?");
        let status = t.get("status").and_then(Value::as_str).unwrap_or("?");
        let wave = t.get("wave").and_then(Value::as_i64).unwrap_or(0);
        let sequence = t.get("sequence").and_then(Value::as_i64).unwrap_or(0);
        let title = t.get("title").and_then(Value::as_str).unwrap_or("");
        let title_short: String = title.chars().take(60).collect();
        println!(
            "{}.{:<3} {:<11} {:<8} {}",
            wave, sequence, status, id, title_short
        );
    }
    Ok(())
}
