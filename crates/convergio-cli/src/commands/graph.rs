//! `cvg graph ...` — Tier-3 retrieval client (ADR-0014).
//!
//! Pure HTTP. The daemon owns the SQLite store and the syn parser;
//! the CLI just renders.

use super::{Client, OutputMode};
use anyhow::Result;
use clap::Subcommand;
use serde_json::Value;

/// Graph subcommands.
#[derive(Subcommand)]
pub enum GraphCommand {
    /// Walk the workspace and refresh the graph in the daemon's SQLite.
    Build {
        /// Repo root (defaults to daemon's cwd).
        #[arg(long)]
        manifest_dir: Option<String>,
        /// Re-parse every file even if mtime unchanged.
        #[arg(long)]
        force: bool,
    },
    /// Print the current node + edge counts.
    Stats,
    /// Emit a context-pack scoped to one task.
    ForTask {
        /// Task id.
        task_id: String,
        /// Cap on matched-node count.
        #[arg(long)]
        node_limit: Option<usize>,
        /// Cap on the file-union token estimate.
        #[arg(long)]
        token_budget: Option<u64>,
    },
    /// Compare ADR claims (touches_crates) against the actual git
    /// diff. Reports drift (touched but not declared) and ghosts
    /// (declared but not touched).
    Drift {
        /// Git ref to diff against (default `origin/main`).
        #[arg(long)]
        since: Option<String>,
        /// Optional ADR id to scope the declared set.
        #[arg(long)]
        adr: Option<String>,
        /// Repo root (defaults to daemon's cwd).
        #[arg(long)]
        repo_root: Option<String>,
    },
}

/// Entry point.
pub async fn run(client: &Client, output: OutputMode, cmd: GraphCommand) -> Result<()> {
    match cmd {
        GraphCommand::Build {
            manifest_dir,
            force,
        } => build(client, output, manifest_dir, force).await,
        GraphCommand::Stats => stats(client, output).await,
        GraphCommand::ForTask {
            task_id,
            node_limit,
            token_budget,
        } => for_task(client, output, &task_id, node_limit, token_budget).await,
        GraphCommand::Drift {
            since,
            adr,
            repo_root,
        } => drift(client, output, since, adr, repo_root).await,
    }
}

async fn drift(
    client: &Client,
    output: OutputMode,
    since: Option<String>,
    adr: Option<String>,
    repo_root: Option<String>,
) -> Result<()> {
    let mut path = String::from("/v1/graph/drift?");
    if let Some(s) = since {
        path.push_str(&format!("since={s}&"));
    }
    if let Some(a) = adr {
        path.push_str(&format!("adr={a}&"));
    }
    if let Some(r) = repo_root {
        path.push_str(&format!("repo_root={r}&"));
    }
    let report: Value = client.get(&path).await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputMode::Plain => render_plain(&report),
        OutputMode::Human => render_drift_human(&report),
    }
    Ok(())
}

fn render_drift_human(report: &Value) {
    let since = report.get("since").and_then(Value::as_str).unwrap_or("?");
    let adr_scope = report
        .get("adr_scope")
        .and_then(Value::as_str)
        .unwrap_or("(all proposed/accepted ADRs)");
    let files = report
        .get("files_changed")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    println!("Drift report (since {since}, scope: {adr_scope})");
    println!("  files changed: {files}");
    print_set("  actual crates", report.get("actual_crates"));
    print_set("  declared crates", report.get("declared_crates"));
    print_set("  DRIFT (touched but not declared)", report.get("drift"));
    print_set("  ghosts (declared but not touched)", report.get("ghosts"));
}

fn print_set(label: &str, v: Option<&Value>) {
    let items: Vec<&str> = v
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if items.is_empty() {
        println!("{label}: (empty)");
    } else {
        println!("{label}: {}", items.join(", "));
    }
}

async fn build(
    client: &Client,
    output: OutputMode,
    manifest_dir: Option<String>,
    force: bool,
) -> Result<()> {
    let body = serde_json::json!({
        "manifest_dir": manifest_dir,
        "force": force,
    });
    let report: Value = client.post("/v1/graph/build", &body).await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputMode::Plain => render_plain(&report),
        OutputMode::Human => render_build_human(&report),
    }
    Ok(())
}

async fn stats(client: &Client, output: OutputMode) -> Result<()> {
    let body: Value = client.get("/v1/graph/stats").await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
        OutputMode::Plain => render_plain(&body),
        OutputMode::Human => println!(
            "Graph: {} nodes, {} edges.",
            body.get("nodes").and_then(Value::as_u64).unwrap_or(0),
            body.get("edges").and_then(Value::as_u64).unwrap_or(0),
        ),
    }
    Ok(())
}

async fn for_task(
    client: &Client,
    output: OutputMode,
    task_id: &str,
    node_limit: Option<usize>,
    token_budget: Option<u64>,
) -> Result<()> {
    let mut path = format!("/v1/graph/for-task/{task_id}?");
    if let Some(n) = node_limit {
        path.push_str(&format!("node_limit={n}&"));
    }
    if let Some(t) = token_budget {
        path.push_str(&format!("token_budget={t}&"));
    }
    let pack: Value = client.get(&path).await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&pack)?),
        OutputMode::Plain => render_plain(&pack),
        OutputMode::Human => render_pack_human(&pack),
    }
    Ok(())
}

fn render_plain(v: &Value) {
    println!("{}", serde_json::to_string(v).unwrap_or_default());
}

fn render_build_human(report: &Value) {
    let nodes = report.get("nodes").and_then(Value::as_u64).unwrap_or(0);
    let edges = report.get("edges").and_then(Value::as_u64).unwrap_or(0);
    let crates = report.get("crates").and_then(Value::as_u64).unwrap_or(0);
    let parsed = report
        .get("files_parsed")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let skipped = report
        .get("files_skipped")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    println!(
        "Graph build: {crates} crates, {parsed} files parsed ({skipped} skipped). \
         Total: {nodes} nodes / {edges} edges."
    );
}

fn render_pack_human(pack: &Value) {
    let task_id = pack.get("task_id").and_then(Value::as_str).unwrap_or("?");
    let tokens = pack
        .get("query_tokens")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let est = pack
        .get("estimated_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    println!("Context-pack for task {task_id}");
    println!("  query tokens: {tokens}");
    println!("  estimated_tokens: {est}");

    if let Some(nodes) = pack.get("matched_nodes").and_then(Value::as_array) {
        println!("  matched nodes ({}):", nodes.len());
        for n in nodes.iter().take(10) {
            let kind = n.get("kind").and_then(Value::as_str).unwrap_or("");
            let name = n.get("name").and_then(Value::as_str).unwrap_or("");
            let crate_name = n.get("crate_name").and_then(Value::as_str).unwrap_or("");
            let score = n.get("score").and_then(Value::as_u64).unwrap_or(0);
            let file = n
                .get("file_path")
                .and_then(Value::as_str)
                .unwrap_or("(no file)");
            println!("    [{kind}] {name} ({crate_name}) score={score} {file}");
        }
        if nodes.len() > 10 {
            println!(
                "    ... and {} more (use --output json for full list)",
                nodes.len() - 10
            );
        }
    }
    if let Some(files) = pack.get("files").and_then(Value::as_array) {
        println!("  files ({}):", files.len());
        for f in files.iter().take(10) {
            let p = f.get("path").and_then(Value::as_str).unwrap_or("");
            let n = f.get("node_count").and_then(Value::as_u64).unwrap_or(0);
            println!("    {p} ({n} matches)");
        }
    }
    if let Some(adrs) = pack.get("related_adrs").and_then(Value::as_array) {
        if !adrs.is_empty() {
            println!("  related ADRs:");
            for a in adrs {
                let id = a.get("adr_id").and_then(Value::as_str).unwrap_or("");
                let via = a.get("via_crate").and_then(Value::as_str).unwrap_or("");
                let f = a.get("file_path").and_then(Value::as_str).unwrap_or("");
                println!("    {id} (via {via}) — {f}");
            }
        }
    }
}
