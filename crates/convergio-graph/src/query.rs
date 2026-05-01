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

/// Compute a [`ContextPack`] from arbitrary text. `task_id` is
/// echoed but not used in the matching itself.
pub async fn for_task_text(
    store: &Store,
    task_id: &str,
    text: &str,
    node_limit: usize,
    token_budget: u64,
) -> Result<ContextPack> {
    let tokens = tokenise(text);
    let mut scored: BTreeMap<String, (i64, MatchedNode)> = BTreeMap::new();

    for token in &tokens {
        let pat = format!("%{}%", token.to_ascii_lowercase());
        let rows = sqlx::query(
            "SELECT id, kind, name, crate_name, file_path \
             FROM graph_nodes \
             WHERE LOWER(name) LIKE ? AND kind != 'adr' AND kind != 'doc'",
        )
        .bind(&pat)
        .fetch_all(store.pool().inner())
        .await?;

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
    matched.sort_by(|a, b| b.score.cmp(&a.score).then(a.name.cmp(&b.name)));
    matched.truncate(node_limit);

    let files = aggregate_files(&matched);
    let estimated_tokens = estimate_tokens(&files, token_budget).await;
    let related_adrs = related_adrs_for(store, &matched).await?;

    Ok(ContextPack {
        task_id: task_id.to_string(),
        query_tokens: tokens,
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

async fn estimate_tokens(files: &[MatchedFile], _budget: u64) -> u64 {
    // Sum file sizes / 4 — same heuristic the LLM clients use.
    let mut total: u64 = 0;
    for f in files {
        if let Ok(meta) = std::fs::metadata(&f.path) {
            total += meta.len() / 4;
        }
    }
    total
}

async fn related_adrs_for(store: &Store, matched: &[MatchedNode]) -> Result<Vec<RelatedAdr>> {
    use std::collections::BTreeSet;
    let crates: BTreeSet<&str> = matched.iter().map(|n| n.crate_name.as_str()).collect();
    if crates.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = crates.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    // Find ADR/Doc nodes that have a `claims` edge to any crate node
    // for one of `crates`.
    let sql = format!(
        "SELECT n.name AS adr_id, n.file_path AS file_path, c.crate_name AS via_crate \
         FROM graph_edges e \
         JOIN graph_nodes n ON e.src = n.id \
         JOIN graph_nodes c ON e.dst = c.id \
         WHERE e.kind = 'claims' AND c.kind = 'crate' AND c.crate_name IN ({placeholders})"
    );
    let mut q = sqlx::query(&sql);
    for c in &crates {
        q = q.bind(*c);
    }
    let rows = q.fetch_all(store.pool().inner()).await?;
    let mut out: Vec<RelatedAdr> = Vec::new();
    for row in rows {
        let adr_id: String = row.try_get("adr_id")?;
        let file_path: Option<String> = row.try_get("file_path")?;
        let via_crate: String = row.try_get("via_crate")?;
        if let Some(fp) = file_path {
            out.push(RelatedAdr {
                adr_id,
                file_path: fp,
                via_crate,
            });
        }
    }
    out.sort_by(|a, b| a.adr_id.cmp(&b.adr_id).then(a.via_crate.cmp(&b.via_crate)));
    Ok(out)
}
