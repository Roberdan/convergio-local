//! Integration tests for graph context-pack ranking.

use convergio_db::Pool;
use convergio_graph::{for_task_text, Node, NodeKind, Store, DEFAULT_TOKEN_BUDGET};

async fn migrated_store() -> anyhow::Result<(tempfile::TempDir, Store)> {
    let dir = tempfile::tempdir()?;
    let url = format!("sqlite://{}?mode=rwc", dir.path().join("g.db").display());
    let pool = Pool::connect(&url).await?;
    let store = Store::new(pool);
    store.migrate().await?;
    Ok((dir, store))
}

fn node(id: &str, kind: NodeKind, name: &str, crate_name: &str, file_path: Option<&str>) -> Node {
    Node {
        id: id.into(),
        kind,
        name: name.into(),
        file_path: file_path.map(str::to_string),
        crate_name: crate_name.into(),
        item_kind: None,
        span: None,
    }
}

#[tokio::test]
async fn ranks_matches_by_kind_score_then_name() -> anyhow::Result<()> {
    let (_dir, store) = migrated_store().await?;
    for n in [
        node(
            "item",
            NodeKind::Item,
            "graph_item",
            "x",
            Some("src/lib.rs"),
        ),
        node(
            "module",
            NodeKind::Module,
            "graph_module",
            "x",
            Some("src/lib.rs"),
        ),
        node("crate", NodeKind::Crate, "graph_crate", "graph_crate", None),
    ] {
        store.upsert_node(&n).await?;
    }

    let pack = for_task_text(&store, "t1", "graph", 10, DEFAULT_TOKEN_BUDGET).await?;
    let names: Vec<_> = pack.matched_nodes.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(names, vec!["graph_crate", "graph_module", "graph_item"]);
    Ok(())
}

#[tokio::test]
async fn token_budget_limits_returned_files() -> anyhow::Result<()> {
    let (_dir, store) = migrated_store().await?;
    for n in [
        node(
            "lib",
            NodeKind::Item,
            "budgetmatch_lib",
            "x",
            Some("src/lib.rs"),
        ),
        node(
            "model",
            NodeKind::Item,
            "budgetmatch_model",
            "x",
            Some("src/model.rs"),
        ),
    ] {
        store.upsert_node(&n).await?;
    }

    let pack = for_task_text(&store, "t1", "budgetmatch", 10, 1).await?;
    assert_eq!(pack.matched_nodes.len(), 2);
    assert!(pack.files.is_empty());
    assert!(pack.estimated_tokens <= 1);
    Ok(())
}
