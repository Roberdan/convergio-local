//! SQLite persistence for graph nodes + edges.
//!
//! All writes happen in a single transaction per call so that a
//! failed parse mid-build does not leave a partial graph.

use crate::error::Result;
use crate::model::{Edge, EdgeKind, Node, NodeKind};
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use sqlx::{Row, Sqlite, Transaction};
use std::collections::HashMap;
use std::path::Path;

// Migration range 600-699 reserved for convergio-graph (ADR-0003).
// `set_ignore_missing(true)` in `migrate()` lets this migrator
// coexist with sibling crates on the same `_sqlx_migrations` table.

/// Storage handle: thin wrapper around the shared SQLite pool.
#[derive(Clone)]
pub struct Store {
    pool: Pool,
}

impl Store {
    /// Bind to the existing SQLite pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Run pending migrations (range 600-699). Idempotent — safe to
    /// call on every daemon start. Coexists with sibling crates'
    /// migrators thanks to `set_ignore_missing(true)`.
    pub async fn migrate(&self) -> Result<()> {
        let mut migrator = sqlx::migrate!("./migrations");
        migrator.set_ignore_missing(true);
        migrator.run(self.pool.inner()).await?;
        Ok(())
    }

    /// Replace all nodes + edges for a given file with the supplied
    /// set, atomically. Stores the file's mtime so subsequent calls
    /// can detect staleness.
    pub async fn upsert_file(
        &self,
        file_path: &str,
        source_mtime: DateTime<Utc>,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<()> {
        let mut tx: Transaction<'_, Sqlite> = self.pool.inner().begin().await?;

        // Drop the previous nodes for this file (cascades to edges).
        sqlx::query("DELETE FROM graph_nodes WHERE file_path = ?")
            .bind(file_path)
            .execute(&mut *tx)
            .await?;

        let now = Utc::now().to_rfc3339();
        let mtime = source_mtime.to_rfc3339();
        for n in nodes {
            sqlx::query(
                "INSERT OR REPLACE INTO graph_nodes \
                 (id, kind, name, file_path, crate_name, item_kind, span_start, span_end, last_parsed, source_mtime) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&n.id)
            .bind(n.kind.as_str())
            .bind(&n.name)
            .bind(n.file_path.as_deref())
            .bind(&n.crate_name)
            .bind(n.item_kind)
            .bind(n.span.map(|(s, _)| s as i64))
            .bind(n.span.map(|(_, e)| e as i64))
            .bind(&now)
            .bind(&mtime)
            .execute(&mut *tx)
            .await?;
        }
        for e in edges {
            sqlx::query(
                "INSERT OR REPLACE INTO graph_edges (src, dst, kind, weight) VALUES (?, ?, ?, ?)",
            )
            .bind(&e.src)
            .bind(&e.dst)
            .bind(e.kind.as_str())
            .bind(e.weight as i64)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Insert (or replace) a single non-file-bound node — useful for
    /// crate-level + ADR/doc nodes that don't have a parsed source.
    pub async fn upsert_node(&self, node: &Node) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT OR REPLACE INTO graph_nodes \
             (id, kind, name, file_path, crate_name, item_kind, span_start, span_end, last_parsed, source_mtime) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&node.id)
        .bind(node.kind.as_str())
        .bind(&node.name)
        .bind(node.file_path.as_deref())
        .bind(&node.crate_name)
        .bind(node.item_kind)
        .bind(node.span.map(|(s, _)| s as i64))
        .bind(node.span.map(|(_, e)| e as i64))
        .bind(&now)
        .bind(&now)
        .execute(self.pool.inner())
        .await?;
        Ok(())
    }

    /// Insert (or replace) a single edge.
    pub async fn upsert_edge(&self, edge: &Edge) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO graph_edges (src, dst, kind, weight) VALUES (?, ?, ?, ?)",
        )
        .bind(&edge.src)
        .bind(&edge.dst)
        .bind(edge.kind.as_str())
        .bind(edge.weight as i64)
        .execute(self.pool.inner())
        .await?;
        Ok(())
    }

    /// Total node count.
    pub async fn count_nodes(&self) -> Result<usize> {
        let row = sqlx::query("SELECT COUNT(*) AS c FROM graph_nodes")
            .fetch_one(self.pool.inner())
            .await?;
        let c: i64 = row.try_get("c")?;
        Ok(c as usize)
    }

    /// Total edge count.
    pub async fn count_edges(&self) -> Result<usize> {
        let row = sqlx::query("SELECT COUNT(*) AS c FROM graph_edges")
            .fetch_one(self.pool.inner())
            .await?;
        let c: i64 = row.try_get("c")?;
        Ok(c as usize)
    }

    /// Per-file mtime map for staleness checks. Empty map = first build.
    pub async fn file_mtimes(&self) -> Result<HashMap<String, String>> {
        let rows = sqlx::query(
            "SELECT file_path, MAX(source_mtime) AS m FROM graph_nodes \
             WHERE file_path IS NOT NULL GROUP BY file_path",
        )
        .fetch_all(self.pool.inner())
        .await?;
        let mut out = HashMap::with_capacity(rows.len());
        for row in rows {
            let p: Option<String> = row.try_get("file_path")?;
            let m: Option<String> = row.try_get("m")?;
            if let (Some(p), Some(m)) = (p, m) {
                out.insert(p, m);
            }
        }
        Ok(out)
    }
}

/// Returns true if the on-disk file has a newer mtime than the
/// stored value (or no stored value exists).
pub fn is_stale(file: &Path, stored: Option<&String>) -> bool {
    let Ok(meta) = std::fs::metadata(file) else {
        return true;
    };
    let Ok(modified) = meta.modified() else {
        return true;
    };
    let on_disk: DateTime<Utc> = modified.into();
    match stored {
        None => true,
        Some(s) => match DateTime::parse_from_rfc3339(s) {
            Ok(parsed) => on_disk > parsed.with_timezone(&Utc),
            Err(_) => true,
        },
    }
}

// Re-export for the lib root.
pub use sqlx::Row as _SqlxRow;

/// Convenience helper used by tests + tooling.
pub fn edge_kind_to_str(k: EdgeKind) -> &'static str {
    k.as_str()
}

/// Convenience helper used by tests + tooling.
pub fn node_kind_to_str(k: NodeKind) -> &'static str {
    k.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Node;

    #[tokio::test]
    async fn migrate_creates_tables() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", dir.path().join("g.db").display());
        let pool = Pool::connect(&url).await.unwrap();
        let store = Store::new(pool);
        store.migrate().await.unwrap();
        assert_eq!(store.count_nodes().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn upsert_then_count() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", dir.path().join("g.db").display());
        let pool = Pool::connect(&url).await.unwrap();
        let store = Store::new(pool);
        store.migrate().await.unwrap();

        let n = Node {
            id: "abc".into(),
            kind: NodeKind::Crate,
            name: "test-crate".into(),
            file_path: None,
            crate_name: "test-crate".into(),
            item_kind: None,
            span: None,
        };
        store.upsert_node(&n).await.unwrap();
        assert_eq!(store.count_nodes().await.unwrap(), 1);

        // idempotent
        store.upsert_node(&n).await.unwrap();
        assert_eq!(store.count_nodes().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn upsert_file_replaces_previous() {
        let dir = tempfile::tempdir().unwrap();
        let url = format!("sqlite://{}?mode=rwc", dir.path().join("g.db").display());
        let pool = Pool::connect(&url).await.unwrap();
        let store = Store::new(pool);
        store.migrate().await.unwrap();

        let module = Node {
            id: "m1".into(),
            kind: NodeKind::Module,
            name: "lib".into(),
            file_path: Some("src/lib.rs".into()),
            crate_name: "x".into(),
            item_kind: None,
            span: None,
        };
        let item = Node {
            id: "i1".into(),
            kind: NodeKind::Item,
            name: "Foo".into(),
            file_path: Some("src/lib.rs".into()),
            crate_name: "x".into(),
            item_kind: Some("struct"),
            span: None,
        };
        store
            .upsert_file(
                "src/lib.rs",
                Utc::now(),
                &[module.clone(), item.clone()],
                &[Edge {
                    src: "m1".into(),
                    dst: "i1".into(),
                    kind: EdgeKind::Declares,
                    weight: 1,
                }],
            )
            .await
            .unwrap();
        assert_eq!(store.count_nodes().await.unwrap(), 2);
        assert_eq!(store.count_edges().await.unwrap(), 1);

        // Replace with a smaller set: must drop the old item.
        store
            .upsert_file("src/lib.rs", Utc::now(), &[module], &[])
            .await
            .unwrap();
        assert_eq!(store.count_nodes().await.unwrap(), 1);
        assert_eq!(store.count_edges().await.unwrap(), 0);
    }
}
