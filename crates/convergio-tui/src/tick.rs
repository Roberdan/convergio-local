//! Refresh-tick helpers.
//!
//! The event loop in `lib.rs` owns the actual `tokio::time::interval`
//! — this module only exposes pure helpers that are easy to unit
//! test without a runtime. Today they format the footer's
//! "refresh in N s" hint and clamp tick configuration values.

use std::time::Duration;

/// Clamp a user-supplied tick value into the band the dashboard
/// supports.
pub fn clamp_tick_secs(secs: u64) -> u64 {
    secs.clamp(1, 300)
}

/// Format a [`Duration`] as `"5s"`, `"1m20s"`, `"1h3m"` for the
/// footer.
pub fn fmt_secs_human(d: Duration) -> String {
    let total = d.as_secs();
    if total < 60 {
        return format!("{total}s");
    }
    if total < 3600 {
        let m = total / 60;
        let s = total % 60;
        if s == 0 {
            return format!("{m}m");
        }
        return format!("{m}m{s}s");
    }
    let h = total / 3600;
    let m = (total % 3600) / 60;
    if m == 0 {
        format!("{h}h")
    } else {
        format!("{h}h{m}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_handles_extremes() {
        assert_eq!(clamp_tick_secs(0), 1);
        assert_eq!(clamp_tick_secs(1), 1);
        assert_eq!(clamp_tick_secs(5), 5);
        assert_eq!(clamp_tick_secs(300), 300);
        assert_eq!(clamp_tick_secs(10_000), 300);
    }

    #[test]
    fn fmt_secs_human_under_minute() {
        assert_eq!(fmt_secs_human(Duration::from_secs(0)), "0s");
        assert_eq!(fmt_secs_human(Duration::from_secs(5)), "5s");
        assert_eq!(fmt_secs_human(Duration::from_secs(59)), "59s");
    }

    #[test]
    fn fmt_secs_human_minutes() {
        assert_eq!(fmt_secs_human(Duration::from_secs(60)), "1m");
        assert_eq!(fmt_secs_human(Duration::from_secs(80)), "1m20s");
        assert_eq!(fmt_secs_human(Duration::from_secs(3599)), "59m59s");
    }

    #[test]
    fn fmt_secs_human_hours() {
        assert_eq!(fmt_secs_human(Duration::from_secs(3600)), "1h");
        assert_eq!(fmt_secs_human(Duration::from_secs(3780)), "1h3m");
    }
}
