//! Trusted-local Thor pipeline execution.

use std::{process::Stdio, time::Duration};
use tokio::{process::Command, time::timeout};

/// Trusted-local pipeline command environment variable name.
pub(crate) const PIPELINE_ENV: &str = "CONVERGIO_THOR_PIPELINE_CMD";
/// Default maximum wall-clock runtime for a pipeline command.
pub(crate) const DEFAULT_PIPELINE_TIMEOUT_SECS: u64 = 600;

const PIPELINE_TAIL_BYTES: usize = 4096;

/// Normalized Thor pipeline configuration.
#[derive(Clone)]
pub(crate) struct Config {
    command: String,
    timeout: Duration,
}

/// Return the default pipeline timeout.
pub(crate) fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_PIPELINE_TIMEOUT_SECS)
}

/// Build pipeline configuration from process environment.
pub(crate) fn from_env() -> Option<Config> {
    from_command(std::env::var(PIPELINE_ENV).ok(), default_timeout())
}

/// Build pipeline configuration from an explicit command.
pub(crate) fn from_command(command: Option<String>, timeout: Duration) -> Option<Config> {
    command
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|command| Config { command, timeout })
}

/// Run the configured pipeline and return a verdict reason on failure.
pub(crate) async fn run(pipeline: &Config) -> Option<String> {
    let child = match Command::new("sh")
        .arg("-c")
        .arg(&pipeline.command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            return Some(format!(
                "pipeline `{}` could not be invoked: {e}",
                pipeline.command
            ));
        }
    };

    match timeout(pipeline.timeout, child.wait_with_output()).await {
        Ok(Ok(o)) if o.status.success() => None,
        Ok(Ok(o)) => Some(format!(
            "pipeline `{}` failed (exit={}): {}",
            pipeline.command,
            o.status
                .code()
                .map_or_else(|| "signal".to_string(), |code| code.to_string()),
            output_tail(&o.stdout, &o.stderr)
        )),
        Ok(Err(e)) => Some(format!(
            "pipeline `{}` could not be invoked: {e}",
            pipeline.command
        )),
        Err(_) => Some(format!(
            "pipeline `{}` timed out after {}",
            pipeline.command,
            timeout_label(pipeline.timeout)
        )),
    }
}

fn timeout_label(value: Duration) -> String {
    if value.subsec_millis() == 0 {
        format!("{}s", value.as_secs())
    } else {
        format!("{}ms", value.as_millis())
    }
}

fn output_tail(stdout: &[u8], stderr: &[u8]) -> String {
    let mut output = Vec::with_capacity(stdout.len() + stderr.len());
    output.extend_from_slice(stdout);
    output.extend_from_slice(stderr);
    if output.len() <= PIPELINE_TAIL_BYTES {
        return String::from_utf8_lossy(&output).into_owned();
    }
    let start = output.len() - PIPELINE_TAIL_BYTES;
    format!(
        "[pipeline output truncated; showing last {PIPELINE_TAIL_BYTES} bytes]\n{}",
        String::from_utf8_lossy(&output[start..])
    )
}
