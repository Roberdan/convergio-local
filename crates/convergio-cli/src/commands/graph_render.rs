//! Renderers for `cvg graph ...` (human / plain). JSON output is
//! emitted directly by [`super::graph`]; this module only owns the
//! human-readable layouts.

use serde_json::Value;

/// Compact JSON dump used by `--output plain`.
pub(super) fn render_plain(v: &Value) {
    println!("{}", serde_json::to_string(v).unwrap_or_default());
}

/// Human renderer for `cvg graph build`.
pub(super) fn render_build_human(report: &Value) {
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

/// Human renderer for `cvg graph for-task`.
pub(super) fn render_pack_human(pack: &Value) {
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

/// Human renderer for `cvg graph drift`.
pub(super) fn render_drift_human(report: &Value) {
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

/// Human renderer for `cvg graph cluster`.
pub(super) fn render_cluster_human(report: &Value) {
    let crate_name = report
        .get("crate_name")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let total_files = report
        .get("total_files")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total_items = report
        .get("total_items")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total_loc = report.get("total_loc").and_then(Value::as_u64).unwrap_or(0);
    let target = report.get("target_loc").and_then(Value::as_u64);
    let above = report
        .get("above_target")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let iters = report
        .get("iterations")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let comms = report
        .get("communities")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    println!(
        "Cluster report for {crate_name}: {total_files} files, {total_items} items, {total_loc} LOC, \
         {} communities (after {iters} iter)",
        comms.len()
    );
    if let Some(t) = target {
        let flag = if above { " — ABOVE TARGET" } else { "" };
        println!("  target LOC: {t}{flag}");
    }
    for (i, c) in comms.iter().enumerate() {
        let label = c.get("label").and_then(Value::as_str).unwrap_or("?");
        let loc = c.get("loc").and_then(Value::as_u64).unwrap_or(0);
        let items = c.get("item_count").and_then(Value::as_u64).unwrap_or(0);
        let internal = c.get("internal_edges").and_then(Value::as_u64).unwrap_or(0);
        let external = c.get("external_edges").and_then(Value::as_u64).unwrap_or(0);
        let files = c
            .get("files")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        println!(
            "  [{i}] community {label}: {} files / {items} items / {loc} LOC \
             (internal={internal}, external={external})",
            files.len()
        );
        for f in files.iter().take(5) {
            if let Some(p) = f.as_str() {
                println!("        {p}");
            }
        }
        if files.len() > 5 {
            println!("        ... and {} more", files.len() - 5);
        }
    }
}
