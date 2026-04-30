//! Output renderers for `cvg pr stack`.
//!
//! Split out of `pr.rs` to keep both files under the 300-line cap.
//! Three modes per the CLI's global `--output` flag:
//!
//! - `human` (default): a one-line summary per PR plus a
//!   "Suggested merge order" arrow chain.
//! - `json`: structured output for piping into other tools.
//! - `plain`: bare PR numbers, in the suggested merge order, one
//!   per line — designed for shell pipelines like
//!   `for pr in $(cvg pr stack --output plain); do gh pr view $pr; done`.

use super::pr::AnalysedPr;
use super::OutputMode;
use anyhow::Result;

pub(crate) fn render(output: OutputMode, prs: &[AnalysedPr], order: &[i64]) -> Result<()> {
    match output {
        OutputMode::Plain => {
            for n in order {
                println!("{n}");
            }
        }
        OutputMode::Json => {
            let value = serde_json::json!({
                "prs": prs.iter().map(|p| serde_json::json!({
                    "number": p.number,
                    "title": p.title,
                    "files": p.files,
                    "depends_on": p.depends_on,
                })).collect::<Vec<_>>(),
                "suggested_order": order,
            });
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        OutputMode::Human => render_human(prs, order),
    }
    Ok(())
}

fn render_human(prs: &[AnalysedPr], order: &[i64]) {
    if prs.is_empty() {
        println!("(no open PRs)");
        return;
    }
    println!("Open PRs ({}):", prs.len());
    for p in prs {
        let n_files = p.files.len();
        let manifest = if n_files > 0 {
            format!("{n_files} file(s)")
        } else {
            "(no Files-touched manifest)".to_string()
        };
        let deps = if p.depends_on.is_empty() {
            String::new()
        } else {
            format!(
                " depends-on=[{}]",
                p.depends_on
                    .iter()
                    .map(|n| format!("#{n}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        let conflicts = compute_conflicts(p, prs);
        let conflicts_str = if conflicts.is_empty() {
            String::new()
        } else {
            format!(
                " conflicts-with=[{}]",
                conflicts
                    .iter()
                    .map(|n| format!("#{n}"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        println!(
            "  #{:<4} {} ({}){}{}",
            p.number,
            truncate(&p.title, 60),
            manifest,
            deps,
            conflicts_str
        );
    }
    println!();
    print!("Suggested merge order: ");
    println!(
        "{}",
        order
            .iter()
            .map(|n| format!("#{n}"))
            .collect::<Vec<_>>()
            .join(" -> ")
    );
}

fn compute_conflicts(target: &AnalysedPr, all: &[AnalysedPr]) -> Vec<i64> {
    let mut out = Vec::new();
    for p in all {
        if p.number == target.number {
            continue;
        }
        if target.files.intersection(&p.files).next().is_some() {
            out.push(p.number);
        }
    }
    out
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut buf: String = s.chars().take(n.saturating_sub(1)).collect();
        buf.push('…');
        buf
    }
}
