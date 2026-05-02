//! Node matching helpers for graph context packs.

use crate::error::Result;
use crate::query::MatchedNode;
use crate::store::Store;
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};

const MAX_ROWS_PER_TOKEN: i64 = 500;

/// Find graph nodes for a token, optionally constrained to crate names.
pub async fn query_token_matches(
    store: &Store,
    token: &str,
    scope: &BTreeSet<String>,
) -> Result<Vec<sqlx::sqlite::SqliteRow>> {
    let pat = format!("%{}%", token.to_ascii_lowercase());
    if scope.is_empty() {
        return Ok(sqlx::query(
            "SELECT id, kind, name, crate_name, file_path \
             FROM graph_nodes \
             WHERE LOWER(name) LIKE ? AND kind != 'adr' AND kind != 'doc' \
             ORDER BY CASE kind WHEN 'crate' THEN 0 WHEN 'module' THEN 1 WHEN 'item' THEN 2 ELSE 3 END, \
                       LOWER(name), id \
             LIMIT ?",
        )
        .bind(&pat)
        .bind(MAX_ROWS_PER_TOKEN)
        .fetch_all(store.pool().inner())
        .await?);
    }

    let placeholders = scope.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, kind, name, crate_name, file_path \
         FROM graph_nodes \
         WHERE LOWER(name) LIKE ? AND kind != 'adr' AND kind != 'doc' \
           AND crate_name IN ({placeholders}) \
         ORDER BY CASE kind WHEN 'crate' THEN 0 WHEN 'module' THEN 1 WHEN 'item' THEN 2 ELSE 3 END, \
                  LOWER(name), id \
         LIMIT ?"
    );
    let mut q = sqlx::query(&sql).bind(&pat);
    for c in scope {
        q = q.bind(c);
    }
    Ok(q.bind(MAX_ROWS_PER_TOKEN)
        .fetch_all(store.pool().inner())
        .await?)
}

/// Seed scores with explicit crate nodes from structured metadata.
pub async fn score_explicit_crates(
    store: &Store,
    scope: &BTreeSet<String>,
    scored: &mut BTreeMap<String, (i64, MatchedNode)>,
) -> Result<()> {
    for crate_name in scope {
        let row = sqlx::query(
            "SELECT id, kind, name, crate_name, file_path \
             FROM graph_nodes WHERE kind = 'crate' AND crate_name = ? LIMIT 1",
        )
        .bind(crate_name)
        .fetch_optional(store.pool().inner())
        .await?;
        if let Some(row) = row {
            let id: String = row.try_get("id")?;
            scored.insert(
                id.clone(),
                (
                    50,
                    MatchedNode {
                        id,
                        kind: row.try_get("kind")?,
                        name: row.try_get("name")?,
                        crate_name: row.try_get("crate_name")?,
                        file_path: row.try_get("file_path")?,
                        score: 50,
                    },
                ),
            );
        }
    }
    Ok(())
}
