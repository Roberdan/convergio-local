//! Structured hints for graph context-pack generation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Structured metadata that scopes and annotates a graph context pack.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuredContextMetadata {
    /// Primary crate the task is about.
    #[serde(rename = "crate")]
    pub primary_crate: Option<String>,
    /// Additional crates that should be considered related.
    pub related_crates: Vec<String>,
    /// ADR ids or paths explicitly required by the task.
    pub adr_required: Vec<String>,
    /// Documentation paths explicitly required by the task.
    pub docs_required: Vec<String>,
    /// Named validation profile requested by the task.
    pub validation_profile: Option<String>,
}

impl StructuredContextMetadata {
    /// Parse simple `key: value` metadata lines out of task text.
    pub fn from_task_text(text: &str) -> Self {
        let mut out = Self::default();
        for line in text.lines() {
            let Some((raw_key, raw_value)) = line.split_once(':') else {
                continue;
            };
            let key = raw_key.trim().to_ascii_lowercase();
            let value = raw_value.trim();
            if value.is_empty() {
                continue;
            }
            match key.as_str() {
                "crate" | "primary_crate" => out.primary_crate = Some(value.to_string()),
                "crates" | "related_crates" | "related_crate" => {
                    out.related_crates.extend(split_list(value));
                }
                "adr_required" | "adrs_required" | "required_adrs" => {
                    out.adr_required.extend(split_list(value));
                }
                "docs_required" | "required_docs" | "doc_required" => {
                    out.docs_required.extend(split_list(value));
                }
                "validation_profile" | "validation" => {
                    out.validation_profile = Some(value.to_string());
                }
                _ => {}
            }
        }
        out.normalized()
    }

    /// Merge query-string metadata over task-text metadata.
    pub fn merged_with(mut self, other: Self) -> Self {
        if other.primary_crate.is_some() {
            self.primary_crate = other.primary_crate;
        }
        if other.validation_profile.is_some() {
            self.validation_profile = other.validation_profile;
        }
        self.related_crates.extend(other.related_crates);
        self.adr_required.extend(other.adr_required);
        self.docs_required.extend(other.docs_required);
        self.normalized()
    }

    /// Crates that should scope graph matching.
    pub fn crate_scope(&self) -> BTreeSet<String> {
        self.primary_crate
            .iter()
            .chain(self.related_crates.iter())
            .map(|s| s.to_string())
            .collect()
    }

    fn normalized(mut self) -> Self {
        self.primary_crate = trim_nonempty(self.primary_crate);
        self.related_crates = dedupe(self.related_crates);
        self.adr_required = dedupe(self.adr_required);
        self.docs_required = dedupe(self.docs_required);
        self.validation_profile = trim_nonempty(self.validation_profile);
        self
    }
}

fn trim_nonempty(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split([',', ' '])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_task_metadata_lines() {
        let meta = StructuredContextMetadata::from_task_text(
            "Do graph work\ncrate: convergio-graph\nrelated_crates: convergio-server, convergio-cli\nadr_required: 0014 0030\ndocs_required: README.md docs/adr/0014-code-graph-tier3-retrieval.md\nvalidation_profile: graph",
        );
        assert_eq!(meta.primary_crate.as_deref(), Some("convergio-graph"));
        assert!(meta
            .related_crates
            .contains(&"convergio-server".to_string()));
        assert!(meta.related_crates.contains(&"convergio-cli".to_string()));
        assert_eq!(meta.adr_required, vec!["0014", "0030"]);
        assert_eq!(meta.validation_profile.as_deref(), Some("graph"));
    }
}
