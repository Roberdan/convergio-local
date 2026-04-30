//! `cvg demo` — guided local quickstart.
//!
//! The two fixture diffs below are intentionally crafted to exercise the
//! gate pipeline. `DIRTY_DIFF` carries a `TODO` marker that `NoDebtGate`
//! must refuse; `CLEAN_DIFF` is debt-free. Lifting them to `const` makes
//! the intent explicit so a future code-scanning gate over our own `src/`
//! does not flag them as real debt.
use super::Client;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

/// Intentional fixture: contains a `TODO` so `NoDebtGate` refuses the
/// `submitted` transition. Not real debt.
const DIRTY_DIFF: &str = "// TODO: wire this later\nfn handler() {}";

/// Intentional fixture: a debt-free diff used in the clean-path branch.
const CLEAN_DIFF: &str = "fn handler() -> &'static str { \"done\" }";

/// Run a guided local demo.
pub async fn run(client: &Client) -> Result<()> {
    println!("Convergio local demo");
    println!("1. Creating a task that should be refused by the gates...");

    let dirty_plan = create_plan(client, "Gate refusal demo").await?;
    let dirty_task = create_task(client, &dirty_plan, "dirty evidence", &["code"]).await?;
    transition(client, &dirty_task, "in_progress", Some("demo-agent")).await?;
    attach(
        client,
        &dirty_task,
        "code",
        json!({ "diff": DIRTY_DIFF }),
        Some(0),
    )
    .await?;

    match transition(client, &dirty_task, "submitted", Some("demo-agent")).await {
        Ok(_) => bail!("dirty task was accepted; expected a gate refusal"),
        Err(e) => println!("Gate refused dirty task as expected: {e}"),
    }

    println!("2. Creating a clean plan that should validate...");
    let clean_plan = create_plan(client, "Clean local demo").await?;
    let clean_task = create_task(client, &clean_plan, "clean evidence", &["code", "test"]).await?;
    transition(client, &clean_task, "in_progress", Some("demo-agent")).await?;
    attach(
        client,
        &clean_task,
        "code",
        json!({ "diff": CLEAN_DIFF }),
        Some(0),
    )
    .await?;
    attach(
        client,
        &clean_task,
        "test",
        json!({"warnings_count": 0, "errors_count": 0, "failures": []}),
        Some(0),
    )
    .await?;
    transition(client, &clean_task, "submitted", Some("demo-agent")).await?;

    // Per ADR-0011: `done` is set only by Thor as a side-effect of
    // validate. The demo therefore submits, then validates — the agent
    // never self-promotes.
    let verdict: Value = client
        .post(&format!("/v1/plans/{clean_plan}/validate"), &json!({}))
        .await?;
    let audit: Value = client.get("/v1/audit/verify").await?;

    println!("Clean plan id: {clean_plan}");
    println!("Clean task id: {clean_task}");
    println!("Thor verdict:");
    print_json(&verdict)?;
    println!("Audit verification:");
    print_json(&audit)?;
    Ok(())
}

async fn create_plan(client: &Client, title: &str) -> Result<String> {
    let plan: Value = client
        .post(
            "/v1/plans",
            &json!({
                "title": title,
                "description": "Created by cvg demo",
            }),
        )
        .await?;
    id(&plan, "plan")
}

async fn create_task(
    client: &Client,
    plan_id: &str,
    title: &str,
    evidence_required: &[&str],
) -> Result<String> {
    let task: Value = client
        .post(
            &format!("/v1/plans/{plan_id}/tasks"),
            &json!({
                "title": title,
                "evidence_required": evidence_required,
            }),
        )
        .await?;
    id(&task, "task")
}

async fn attach(
    client: &Client,
    task_id: &str,
    kind: &str,
    payload: Value,
    exit_code: Option<i64>,
) -> Result<()> {
    let _: Value = client
        .post(
            &format!("/v1/tasks/{task_id}/evidence"),
            &json!({
                "kind": kind,
                "payload": payload,
                "exit_code": exit_code,
            }),
        )
        .await?;
    Ok(())
}

async fn transition(
    client: &Client,
    task_id: &str,
    target: &str,
    agent_id: Option<&str>,
) -> Result<Value> {
    client
        .post(
            &format!("/v1/tasks/{task_id}/transition"),
            &json!({
                "target": target,
                "agent_id": agent_id,
            }),
        )
        .await
}

fn id(value: &Value, label: &str) -> Result<String> {
    value
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .with_context(|| format!("{label} response did not include id: {value}"))
}

fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
