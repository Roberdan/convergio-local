//! Per-crate AUTO-block generator. Split out of
//! [`super::docs_generators`] so the parent stays under the 300-line
//! cap.
//!
//! `gen_crate_stats` walks up from the markdown file currently being
//! rewritten (e.g. `crates/convergio-graph/AGENTS.md`) until it
//! finds a `Cargo.toml` with a `[package]` block, then summarises
//! that crate's `src/` tree.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Stats for the crate that owns the markdown file being rewritten.
/// Falls back to a generic message when the file is not under a
/// crate directory.
pub(super) fn gen_crate_stats(file_path: &Path, _root: &Path) -> Result<String> {
    let crate_dir = match find_crate_dir(file_path) {
        Some(d) => d,
        None => {
            return Ok(
                "_(no surrounding crate found — `crate_stats` is intended for per-crate AGENTS.md)_\n"
                    .into(),
            );
        }
    };
    let src_dir = crate_dir.join("src");
    let mut total_files: usize = 0;
    let mut total_items: usize = 0;
    let mut total_lines: u64 = 0;
    let mut near_cap: Vec<(String, usize)> = Vec::new();
    if src_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&src_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if !p.is_file() || p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(p) else {
                continue;
            };
            let lines = src.lines().count();
            total_files += 1;
            total_lines += lines as u64;
            for line in src.lines() {
                if is_public_item(line.trim_start()) {
                    total_items += 1;
                }
            }
            if lines >= 250 {
                let rel = p
                    .strip_prefix(&crate_dir)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .into_owned();
                near_cap.push((rel, lines));
            }
        }
    }
    near_cap.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let crate_name = crate_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("(unknown)");
    let mut out = String::new();
    out.push_str(&format!(
        "**`{crate_name}` stats:** {total_files} `*.rs` files / {total_items} public items / {total_lines} lines (under `src/`).\n",
    ));
    if near_cap.is_empty() {
        out.push_str("\nNo files within 50 lines of the 300-line cap.\n");
    } else {
        out.push_str("\nFiles approaching the 300-line cap:\n");
        for (path, lines) in near_cap {
            out.push_str(&format!("- `{path}` ({lines} lines)\n"));
        }
    }
    Ok(out)
}

fn is_public_item(t: &str) -> bool {
    t.starts_with("pub fn ")
        || t.starts_with("pub struct ")
        || t.starts_with("pub enum ")
        || t.starts_with("pub trait ")
        || t.starts_with("pub const ")
        || t.starts_with("pub type ")
        || t.starts_with("pub use ")
}

fn find_crate_dir(file_path: &Path) -> Option<PathBuf> {
    let mut current = file_path.parent()?;
    loop {
        let manifest = current.join("Cargo.toml");
        if manifest.is_file() {
            if let Ok(src) = std::fs::read_to_string(&manifest) {
                if src.contains("[package]") {
                    return Some(current.to_path_buf());
                }
            }
        }
        current = current.parent()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_public_item_recognises_common_pubs() {
        assert!(is_public_item("pub fn foo() {}"));
        assert!(is_public_item("pub struct Bar;"));
        assert!(is_public_item("pub enum Baz {"));
        assert!(is_public_item("pub trait T {"));
        assert!(!is_public_item("fn private() {}"));
        assert!(!is_public_item("pub(crate) fn x() {}"));
    }

    #[test]
    fn find_crate_dir_walks_up_to_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let crate_dir = dir.path().join("my-crate");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"my-crate\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        let nested = crate_dir.join("AGENTS.md");
        std::fs::write(&nested, "x").unwrap();
        let found = find_crate_dir(&nested).unwrap();
        assert_eq!(found, crate_dir);
    }

    #[test]
    fn find_crate_dir_returns_none_outside_a_crate() {
        let dir = tempfile::tempdir().unwrap();
        let lone = dir.path().join("README.md");
        std::fs::write(&lone, "x").unwrap();
        assert!(find_crate_dir(&lone).is_none());
    }
}
