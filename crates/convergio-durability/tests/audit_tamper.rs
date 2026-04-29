//! Audit tamper-detection tests.
//!
//! These tests prove the ADR-0002 security claim: any mutation of a
//! row in `audit_log` (payload edit, hash edit, prev_hash edit, row
//! deletion) is detected by [`AuditLog::verify`].
//!
//! These tests are load-bearing for the product positioning. If they
//! ever go red, **stop and investigate** — the durability story is
//! broken.

use convergio_db::Pool;
use convergio_durability::{init, Durability, NewPlan};
use tempfile::tempdir;

async fn fresh_dur_with_some_history() -> (Durability, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let url = format!("sqlite://{}/state.db", dir.path().display());
    let pool = Pool::connect(&url).await.unwrap();
    init(&pool).await.unwrap();
    let dur = Durability::new(pool);

    // Produce a few audit rows.
    for i in 0..3 {
        dur.create_plan(NewPlan {
            title: format!("plan {i}"),
            description: None,
        })
        .await
        .unwrap();
    }
    (dur, dir)
}

#[tokio::test]
async fn clean_chain_verifies() {
    let (dur, _dir) = fresh_dur_with_some_history().await;
    let report = dur.audit().verify(None, None).await.unwrap();
    assert!(report.ok, "fresh chain must verify clean: {report:?}");
    assert!(report.checked >= 3);
    assert_eq!(report.broken_at, None);
}

#[tokio::test]
async fn payload_mutation_breaks_chain() {
    let (dur, _dir) = fresh_dur_with_some_history().await;

    // Tamper: rewrite the payload at seq=2 without updating the hash.
    sqlx::query("UPDATE audit_log SET payload = ? WHERE seq = ?")
        .bind(r#"{"hacked":true}"#)
        .bind(2_i64)
        .execute(dur.pool().inner())
        .await
        .unwrap();

    let report = dur.audit().verify(None, None).await.unwrap();
    assert!(!report.ok, "mutated payload must NOT verify");
    assert_eq!(report.broken_at, Some(2));
}

#[tokio::test]
async fn hash_mutation_breaks_chain() {
    let (dur, _dir) = fresh_dur_with_some_history().await;

    // Tamper: rewrite the hash at seq=1 to anything.
    sqlx::query("UPDATE audit_log SET hash = ? WHERE seq = ?")
        .bind("deadbeef".repeat(8)) // 64 hex chars
        .bind(1_i64)
        .execute(dur.pool().inner())
        .await
        .unwrap();

    let report = dur.audit().verify(None, None).await.unwrap();
    assert!(!report.ok);
    // The verifier may detect at seq=1 (hash recompute mismatch) or at
    // seq=2 (prev_hash chain break). Either is correct.
    assert!(matches!(report.broken_at, Some(1) | Some(2)));
}

#[tokio::test]
async fn prev_hash_mutation_breaks_chain() {
    let (dur, _dir) = fresh_dur_with_some_history().await;

    // Tamper: rewrite the prev_hash at seq=2.
    sqlx::query("UPDATE audit_log SET prev_hash = ? WHERE seq = ?")
        .bind("0".repeat(64))
        .bind(2_i64)
        .execute(dur.pool().inner())
        .await
        .unwrap();

    let report = dur.audit().verify(None, None).await.unwrap();
    assert!(!report.ok);
    assert_eq!(report.broken_at, Some(2));
}

#[tokio::test]
async fn row_deletion_breaks_chain() {
    let (dur, _dir) = fresh_dur_with_some_history().await;

    // Tamper: delete seq=2 entirely. Verification should detect that
    // seq=3's prev_hash no longer matches seq=1's hash.
    sqlx::query("DELETE FROM audit_log WHERE seq = ?")
        .bind(2_i64)
        .execute(dur.pool().inner())
        .await
        .unwrap();

    let report = dur.audit().verify(None, None).await.unwrap();
    assert!(!report.ok, "deleted row must be detected");
    // The first surviving row whose chain doesn't link is seq=3.
    assert_eq!(report.broken_at, Some(3));
}

#[tokio::test]
async fn ranged_verify_catches_tamper_inside_range() {
    let (dur, _dir) = fresh_dur_with_some_history().await;
    sqlx::query("UPDATE audit_log SET payload = ? WHERE seq = ?")
        .bind(r#"{"x":1}"#)
        .bind(2_i64)
        .execute(dur.pool().inner())
        .await
        .unwrap();

    // [1, 2] catches it.
    let r = dur.audit().verify(Some(1), Some(2)).await.unwrap();
    assert!(!r.ok);
    assert_eq!(r.broken_at, Some(2));

    // [3, ∞) does not see seq=2 directly but seq=3's prev_hash is now
    // computed against the bootstrap (the hash of seq=2, which the
    // verifier loads fresh from the DB) so the chain still links —
    // tampering is invisible OUTSIDE the range. This is the documented
    // limit of ranged verification: cron should run with from=None.
    let r = dur.audit().verify(Some(3), None).await.unwrap();
    assert!(
        r.ok,
        "ranged verify deliberately misses tampering before the range"
    );
}
