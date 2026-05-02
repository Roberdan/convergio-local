//! Read-side queries against the graph store.
//!
//! v0 ships `for_task`: given a free-form task title + description,
//! returns a context-pack JSON with the most relevant code nodes,
//! their files, and any ADR that claims the same crates.
//!
//! Match strategy is intentionally simple (substring + score):
//!   - Tokenise input, drop stopwords + tokens shorter than 3 chars.
//!   - For each remaining token, find graph nodes whose `name`
//!     contains the token (case-insensitive).
//!   - Score: `crate` 10, `module` 3, `item` 1.
//!   - Truncate to top-K + estimate token cost from file sizes.
//!
//! Anything more sophisticated (TF-IDF, embeddings, type-resolved
//! call sites) is future work — a graph-only baseline is enough to
//! demonstrate Tier-3 value over Tier-1/2.

use crate::error::Result;
use crate::query_adrs::related_adrs_for_with_required;
use crate::query_hints::StructuredContextMetadata;
use crate::query_match::{query_token_matches, score_explicit_crates};
use crate::store::Store;
use crate::tokens::tokenise;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::BTreeMap;

/// Aggregate response for `cvg graph for-task`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPack {
    /// Echoed task identifier — empty when called with raw text.
    pub task_id: String,
    /// Tokens extracted from the task text after stopword filtering.
    pub query_tokens: Vec<String>,
    /// Structured task metadata used to scope this pack.
    pub structured_metadata: StructuredContextMetadata,
    /// Top-scored code nodes (crate / module / item) sorted by score desc.
    pub matched_nodes: Vec<MatchedNode>,
    /// Files referenced by the matched nodes, deduplicated.
    pub files: Vec<MatchedFile>,
    /// ADRs that claim crates touched by the matches.
    pub related_adrs: Vec<RelatedAdr>,
    /// Rough token estimate for the union of `files`. 1 token ≈ 4 bytes.
    pub estimated_tokens: u64,
}

/// One code node that matched a query token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedNode {
    /// Stable node id.
    pub id: String,
    /// `crate` | `module` | `item`.
    pub kind: String,
    /// Display name.
    pub name: String,
    /// Owning crate.
    pub crate_name: String,
    /// File path (relative to repo root) — None for crate-only nodes.
    pub file_path: Option<String>,
    /// Aggregated score across query tokens.
    pub score: u32,
}

/// A file mentioned by at least one [`MatchedNode`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedFile {
    /// Relative path.
    pub path: String,
    /// Owning crate.
    pub crate_name: String,
    /// Number of matched nodes inside this file.
    pub node_count: u32,
}

/// One ADR that claims a crate touched by the matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedAdr {
    /// ADR id (e.g. `0014`).
    pub adr_id: String,
    /// Path to the ADR file.
    pub file_path: String,
    /// Crate name that triggered the relation (matched ∩ claimed).
    pub via_crate: String,
}

/// Default cap on matched nodes returned.
pub const DEFAULT_NODE_LIMIT: usize = 25;

/// Default token budget for the file union.
pub const DEFAULT_TOKEN_BUDGET: u64 = 8_000;

/// Hard cap on caller-provided node limits.
pub const MAX_NODE_LIMIT: usize = 100;

/// Hard cap on caller-provided file token budgets.
pub const MAX_TOKEN_BUDGET: u64 = 64_000;

/// Compute a [`ContextPack`] from arbitrary text. `task_id` is
/// echoed but not used in the matching itself.
pub async fn for_task_text(
    store: &Store,
    task_id: &str,
    text: &str,
    node_limit: usize,
    token_budget: u64,
) -> Result<ContextPack> {
    let metadata = StructuredContextMetadata::from_task_text(text);
    for_task_text_with_metadata(store, task_id, text, metadata, node_limit, token_budget).await
}

/// Compute a [`ContextPack`] with explicit caller-supplied metadata.
pub async fn for_task_text_with_metadata(
    store: &Store,
    task_id: &str,
    text: &str,
    metadata: StructuredContextMetadata,
    node_limit: usize,
    token_budget: u64,
) -> Result<ContextPack> {
    let tokens = tokenise(text);
    let node_limit = node_limit.min(MAX_NODE_LIMIT);
    let token_budget = token_budget.min(MAX_TOKEN_BUDGET);
    let scope = metadata.crate_scope();
    let mut scored: BTreeMap<String, (i64, MatchedNode)> = BTreeMap::new();
    score_explicit_crates(store, &scope, &mut scored).await?;

    for token in &tokens {
        let rows = query_token_matches(store, token, &scope).await?;

        for row in rows {
            let id: String = row.try_get("id")?;
            let kind: String = row.try_get("kind")?;
            let name: String = row.try_get("name")?;
            let crate_name: String = row.try_get("crate_name")?;
            let file_path: Option<String> = row.try_get("file_path")?;

            let bump = score_for_kind(&kind);
            let entry = scored.entry(id.clone()).or_insert_with(|| {
                (
                    0,
                    MatchedNode {
                        id: id.clone(),
                        kind: kind.clone(),
                        name: name.clone(),
                        crate_name: crate_name.clone(),
                        file_path: file_path.clone(),
                        score: 0,
                    },
                )
            });
            entry.0 += bump;
            entry.1.score = entry.0 as u32;
        }
    }

    let mut matched: Vec<MatchedNode> = scored.into_values().map(|(_, n)| n).collect();
    matched.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.name.cmp(&b.name))
            .then(a.id.cmp(&b.id))
    });
    matched.truncate(node_limit);

    let (files, estimated_tokens) = apply_token_budget(aggregate_files(&matched), token_budget);
    let related_adrs =
        related_adrs_for_with_required(store, &matched, &metadata.adr_required).await?;

    Ok(ContextPack {
        task_id: task_id.to_string(),
        query_tokens: tokens,
        structured_metadata: metadata,
        matched_nodes: matched,
        files,
        related_adrs,
        estimated_tokens,
    })
}

fn score_for_kind(kind: &str) -> i64 {
    match kind {
        "crate" => 10,
        "module" => 3,
        "item" => 1,
        _ => 0,
    }
}

fn aggregate_files(matched: &[MatchedNode]) -> Vec<MatchedFile> {
    let mut by_path: BTreeMap<String, MatchedFile> = BTreeMap::new();
    for n in matched {
        let Some(path) = n.file_path.as_ref() else {
            continue;
        };
        let entry = by_path.entry(path.clone()).or_insert(MatchedFile {
            path: path.clone(),
            crate_name: n.crate_name.clone(),
            node_count: 0,
        });
        entry.node_count += 1;
    }
    let mut out: Vec<MatchedFile> = by_path.into_values().collect();
    out.sort_by(|a, b| b.node_count.cmp(&a.node_count).then(a.path.cmp(&b.path)));
    out
}

fn apply_token_budget(files: Vec<MatchedFile>, budget: u64) -> (Vec<MatchedFile>, u64) {
    let mut kept = Vec::new();
    let mut total: u64 = 0;
    for f in files {
        let cost = estimate_file_tokens(&f.path);
        if total.saturating_add(cost) > budget {
            continue;
        }
        total += cost;
        kept.push(f);
    }
    (kept, total)
}

fn estimate_file_tokens(path: &str) -> u64 {
    std::fs::metadata(path)
        .map(|meta| meta.len().div_ceil(4))
        .unwrap_or(0)
}
