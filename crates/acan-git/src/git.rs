//! Running the git CLI with a timeout and captured stderr.

use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use tokio::process::Command;

use crate::Error;

/// Upper bound for one git invocation. Local operations finish in well under a
/// second; this only bounds a wedged process, and the child is killed on expiry.
const GIT_TIMEOUT: Duration = Duration::from_secs(30);

/// What a git invocation produced.
pub(crate) struct GitOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

fn render(args: &[&str]) -> String {
    args.join(" ")
}

/// Run git in `dir`, returning whatever it did. A non-zero exit is not an error
/// here; callers that give exit codes meaning inspect `status`.
pub(crate) async fn run(
    dir: &Path,
    args: &[&str],
    envs: &[(String, String)],
) -> Result<GitOutput, Error> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir).args(args);
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    cmd.stdin(Stdio::null());
    cmd.kill_on_drop(true);
    let out = tokio::time::timeout(GIT_TIMEOUT, cmd.output())
        .await
        .map_err(|_| Error::Timeout(render(args)))?
        .map_err(|source| Error::Spawn {
            args: render(args),
            source,
        })?;
    Ok(GitOutput {
        status: out.status,
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// Run git, turning a non-zero exit into [`Error::Git`], and return stdout.
pub(crate) async fn ok(
    dir: &Path,
    args: &[&str],
    envs: &[(String, String)],
) -> Result<String, Error> {
    let out = run(dir, args, envs).await?;
    ensure_ok(out, args)
}

/// Turn a finished invocation into stdout, or [`Error::Git`] on non-zero exit.
pub(crate) fn ensure_ok(out: GitOutput, args: &[&str]) -> Result<String, Error> {
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(Error::Git {
            args: render(args),
            code: out.status.code(),
            stderr: out.stderr.trim().to_owned(),
        })
    }
}
