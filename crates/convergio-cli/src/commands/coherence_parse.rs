//! ADR / index / workspace parsers for [`super::coherence`].
//!
//! Split out to honour the 300-line per-file cap.

use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub(super) struct Adr {
    pub(super) id: String,
    pub(super) path: String,
    pub(super) status: String,
    pub(super) related_adrs: Vec<String>,
    pub(super) touches_crates: Vec<String>,
}

#[derive(Default)]
pub(super) struct Frontmatter {
    pub(super) status: String,
    pub(super) related_adrs: Vec<String>,
    pub(super) touches_crates: Vec<String>,
}

pub(super) fn load_adrs(dir: &Path) -> Result<Vec<Adr>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if n.ends_with(".md") && n != "README.md" => n.to_string(),
            _ => continue,
        };
        let id = name.split('-').next().unwrap_or("").to_string();
        if !id.chars().all(|c| c.is_ascii_digit()) || id.is_empty() {
            continue;
        }
        let body = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let fm = parse_frontmatter(&body).with_context(|| format!("frontmatter {name}"))?;
        out.push(Adr {
            id,
            path: path
                .strip_prefix(dir.parent().unwrap_or(dir))
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned(),
            status: fm.status,
            related_adrs: fm.related_adrs,
            touches_crates: fm.touches_crates,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

pub(super) fn parse_frontmatter(body: &str) -> Result<Frontmatter> {
    let mut lines = body.lines();
    if lines.next().map(str::trim) != Some("---") {
        anyhow::bail!("missing opening --- delimiter");
    }
    let mut fm = Frontmatter::default();
    for line in lines {
        let trimmed = line.trim_end();
        if trimmed.trim() == "---" {
            return Ok(fm);
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "status" => fm.status = value.trim_matches('"').to_string(),
            "related_adrs" => fm.related_adrs = parse_yaml_list(value),
            "touches_crates" => fm.touches_crates = parse_yaml_list(value),
            _ => {}
        }
    }
    anyhow::bail!("missing closing --- delimiter")
}

fn parse_yaml_list(value: &str) -> Vec<String> {
    let inside = value.trim().trim_start_matches('[').trim_end_matches(']');
    if inside.trim().is_empty() {
        return Vec::new();
    }
    inside
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub(super) fn parse_index(path: &Path) -> Result<BTreeMap<String, String>> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut map = BTreeMap::new();
    for line in body.lines() {
        if !line.starts_with("| [") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 4 {
            continue;
        }
        let id_cell = cells[1];
        let status_cell = cells[3];
        let id = id_cell
            .trim_start_matches("[")
            .split(']')
            .next()
            .unwrap_or("")
            .to_string();
        if id.chars().all(|c| c.is_ascii_digit()) && !id.is_empty() {
            map.insert(id, status_cell.to_string());
        }
    }
    Ok(map)
}

pub(super) fn parse_workspace_members(path: &Path) -> Result<BTreeSet<String>> {
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let parsed: toml::Value = body.parse().context("parse Cargo.toml")?;
    let members = parsed
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("workspace.members not found"))?;
    let mut out = BTreeSet::new();
    for m in members {
        if let Some(s) = m.as_str() {
            // Member entries are paths like "crates/convergio-cli".
            let name = s.rsplit('/').next().unwrap_or(s).to_string();
            out.insert(name);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frontmatter_extracts_lists() {
        let body = "---\nid: 0001\nstatus: accepted\nrelated_adrs: [0002, 0003]\ntouches_crates: [convergio-cli]\n---\n# body\n";
        let fm = parse_frontmatter(body).unwrap();
        assert_eq!(fm.status, "accepted");
        assert_eq!(fm.related_adrs, vec!["0002", "0003"]);
        assert_eq!(fm.touches_crates, vec!["convergio-cli"]);
    }

    #[test]
    fn parse_frontmatter_handles_empty_lists() {
        let body = "---\nstatus: proposed\nrelated_adrs: []\ntouches_crates: []\n---\n";
        let fm = parse_frontmatter(body).unwrap();
        assert!(fm.related_adrs.is_empty());
        assert!(fm.touches_crates.is_empty());
    }
}
