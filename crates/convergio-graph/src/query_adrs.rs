//! ADR lookup helpers for graph context packs.

use crate::error::Result;
use crate::query::{MatchedNode, RelatedAdr};
use crate::store::Store;
use sqlx::Row;
use std::collections::BTreeSet;

/// Find ADRs claimed by matched crates and add any required ADRs from metadata.
pub async fn related_adrs_for_with_required(
    store: &Store,
    matched: &[MatchedNode],
    required: &[String],
) -> Result<Vec<RelatedAdr>> {
    let mut out = related_adrs_for(store, matched).await?;
    let mut seen: BTreeSet<(String, String, String)> = out
        .iter()
        .map(|a| (a.adr_id.clone(), a.file_path.clone(), a.via_crate.clone()))
        .collect();
    for req in required {
        for adr in required_adr(store, req).await? {
            let key = (
                adr.adr_id.clone(),
                adr.file_path.clone(),
                adr.via_crate.clone(),
            );
            if seen.insert(key) {
                out.push(adr);
            }
        }
    }
    out.sort_by(|a, b| {
        a.adr_id
            .cmp(&b.adr_id)
            .then(a.via_crate.cmp(&b.via_crate))
            .then(a.file_path.cmp(&b.file_path))
    });
    Ok(out)
}

/// Find ADRs that claim crates touched by matched nodes.
pub async fn related_adrs_for(store: &Store, matched: &[MatchedNode]) -> Result<Vec<RelatedAdr>> {
    let crates: BTreeSet<&str> = matched.iter().map(|n| n.crate_name.as_str()).collect();
    if crates.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = crates.iter().map(|_| "?").collect::<Vec<_>>().join(",");
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
    out.sort_by(|a, b| {
        a.adr_id
            .cmp(&b.adr_id)
            .then(a.via_crate.cmp(&b.via_crate))
            .then(a.file_path.cmp(&b.file_path))
    });
    Ok(out)
}

async fn required_adr(store: &Store, req: &str) -> Result<Vec<RelatedAdr>> {
    let pattern = format!("%{req}%");
    let rows = sqlx::query(
        "SELECT name AS adr_id, file_path \
         FROM graph_nodes \
         WHERE kind = 'adr' AND (name = ? OR file_path = ? OR file_path LIKE ?) \
         ORDER BY name, file_path",
    )
    .bind(req)
    .bind(req)
    .bind(pattern)
    .fetch_all(store.pool().inner())
    .await?;
    let mut out = Vec::new();
    for row in rows {
        let adr_id: String = row.try_get("adr_id")?;
        let file_path: Option<String> = row.try_get("file_path")?;
        if let Some(file_path) = file_path {
            out.push(RelatedAdr {
                adr_id,
                file_path,
                via_crate: "metadata".into(),
            });
        }
    }
    Ok(out)
}
