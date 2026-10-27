//! The worker's local tools. Every path stays inside the task's worktree, and a write is allowed
//! only to a file the task declared. The worker decides when a write may run (see `worker`).

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use acan_llm::ToolSpec;
use serde_json::{Value, json};
use tokio::process::Command;

/// The most bytes a read returns. Longer files are cut and marked.
const MAX_READ_BYTES: usize = 32 * 1024;
/// The most entries a directory listing returns.
const MAX_LISTING: usize = 200;
/// The most bytes of test output the model sees, taken from the end where failures are.
const MAX_TEST_OUTPUT: usize = 4 * 1024;
/// How long one test run may take. A run that hangs ends so the worker can release its task.
const TEST_TIMEOUT: Duration = Duration::from_secs(300);

/// One task's worktree, with the files it declared and the test command it may run.
#[derive(Debug)]
pub struct Worktree {
    root: PathBuf,
    declared: Vec<String>,
    test_command: Option<String>,
}

impl Worktree {
    /// The worktree at `root` for a task that declared `declared`. A declared path that is not
    /// confined to the worktree is dropped, so it can never be written.
    #[must_use]
    pub fn new(root: PathBuf, declared: &[String], test_command: Option<String>) -> Self {
        let declared = declared
            .iter()
            .filter_map(|path| confined(path).ok())
            .collect();
        Self {
            root,
            declared,
            test_command,
        }
    }

    /// The local tools, as the model sees them.
    #[must_use]
    pub fn specs() -> Vec<ToolSpec> {
        vec![
            spec(
                "read_file",
                "Read a file in the worktree.",
                json!({
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                    "required": ["path"],
                }),
            ),
            spec(
                "list_files",
                "List one directory in the worktree. An empty path lists the root.",
                json!({
                    "type": "object",
                    "properties": { "path": { "type": "string" } },
                }),
            ),
            spec(
                "write_file",
                "Write a file you declared. The write waits until the origin clears the task to build.",
                json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" },
                    },
                    "required": ["path", "content"],
                }),
            ),
            spec(
                "run_tests",
                "Run the test command the worker was started with, in the worktree.",
                json!({ "type": "object", "properties": {} }),
            ),
            spec(
                "commit",
                "Commit every change in the worktree. Only declared files may change.",
                json!({
                    "type": "object",
                    "properties": { "message": { "type": "string" } },
                    "required": ["message"],
                }),
            ),
        ]
    }

    /// Reads a file. Its text is cut at a limit and marked when it is longer.
    ///
    /// # Errors
    ///
    /// Returns the reason when the path is not inside the worktree or cannot be read.
    pub async fn read_file(&self, path: &str) -> Result<String, String> {
        let relative = confined(path)?;
        let bytes = tokio::fs::read(self.inside(&relative)?)
            .await
            .map_err(|err| format!("cannot read {relative}: {err}"))?;
        let shown = &bytes[..bytes.len().min(MAX_READ_BYTES)];
        let mut text = String::from_utf8_lossy(shown).into_owned();
        if bytes.len() > MAX_READ_BYTES {
            text.push_str("\n[truncated]");
        }
        Ok(text)
    }

    /// Lists one directory, directories first marked with a trailing slash. `.git` is hidden.
    ///
    /// # Errors
    ///
    /// Returns the reason when the path is not inside the worktree or cannot be listed.
    pub fn list_files(&self, path: &str) -> Result<String, String> {
        let dir = if path.trim().is_empty() || path == "." {
            self.root.clone()
        } else {
            self.inside(&confined(path)?)?
        };
        let entries = std::fs::read_dir(&dir)
            .map_err(|err| format!("cannot list {}: {err}", dir.display()))?;
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                if entry.path().is_dir() {
                    format!("{name}/")
                } else {
                    name
                }
            })
            .filter(|name| name != ".git/" && name != ".git")
            .collect();
        names.sort();
        names.truncate(MAX_LISTING);
        Ok(names.join("\n"))
    }

    /// Writes a declared file, creating its directories.
    ///
    /// # Errors
    ///
    /// Returns the reason when the file is not declared or cannot be written.
    pub fn write_file(&self, path: &str, content: &str) -> Result<String, String> {
        let relative = confined(path)?;
        if !self.declared.contains(&relative) {
            return Err(format!(
                "{relative} is not declared; declared files are: {}",
                self.declared.join(", ")
            ));
        }
        let target = self.inside(&relative)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("cannot create the directory for {relative}: {err}"))?;
        }
        std::fs::write(&target, content)
            .map_err(|err| format!("cannot write {relative}: {err}"))?;
        Ok(format!("wrote {} bytes to {relative}", content.len()))
    }

    /// Runs the configured test command in the worktree and reports its exit and output tail.
    ///
    /// # Errors
    ///
    /// Returns the reason when no test command is configured or it cannot start.
    pub async fn run_tests(&self) -> Result<String, String> {
        let Some(command) = &self.test_command else {
            return Err("no test command is configured for this worker".to_owned());
        };
        let run = shell(command)
            .current_dir(&self.root)
            .kill_on_drop(true)
            .output();
        let output = tokio::time::timeout(TEST_TIMEOUT, run)
            .await
            .map_err(|_| {
                format!(
                    "the test command did not finish within {} seconds",
                    TEST_TIMEOUT.as_secs()
                )
            })?
            .map_err(|err| format!("the test command did not start: {err}"))?;
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        let exit = output
            .status
            .code()
            .map_or_else(|| "by signal".to_owned(), |code| code.to_string());
        Ok(format!("exit {exit}\n{}", tail(&text, MAX_TEST_OUTPUT)))
    }

    /// Stages every change, refuses it when a file outside the declaration changed, and commits.
    /// Returns the new commit id.
    ///
    /// # Errors
    ///
    /// Returns the reason when git fails, when nothing changed, or when a file outside the
    /// declaration changed.
    pub async fn commit(&self, message: &str) -> Result<String, String> {
        git(&self.root, &["add", "-A"]).await?;
        let staged = git(&self.root, &["diff", "--cached", "--name-only"]).await?;
        let outside: Vec<&str> = staged
            .lines()
            .filter(|file| !self.declared.iter().any(|declared| declared == *file))
            .collect();
        if !outside.is_empty() {
            // The refused change stays in the worktree, unstaged, for the worker to see.
            let _ = git(&self.root, &["reset", "-q"]).await;
            return Err(format!(
                "commit refused: {} changed outside the declared files",
                outside.join(", ")
            ));
        }
        git(&self.root, &["commit", "-q", "-m", message]).await?;
        Ok(git(&self.root, &["rev-parse", "HEAD"])
            .await?
            .trim()
            .to_owned())
    }
}

impl Worktree {
    /// The path inside the worktree. Every existing link on the way is resolved, so a symlink
    /// that points outside the worktree is refused, and so is a link that cannot be resolved.
    fn inside(&self, relative: &str) -> Result<PathBuf, String> {
        let joined = self.root.join(relative);
        let root = std::fs::canonicalize(&self.root)
            .map_err(|err| format!("the worktree cannot be resolved: {err}"))?;
        // `symlink_metadata` sees a dangling link too, so the link is resolved and refused.
        let mut probe = joined.as_path();
        while probe.symlink_metadata().is_err() {
            probe = probe
                .parent()
                .ok_or_else(|| format!("{relative} is outside the worktree"))?;
        }
        let real = std::fs::canonicalize(probe)
            .map_err(|err| format!("cannot resolve {relative}: {err}"))?;
        if real.starts_with(&root) {
            Ok(joined)
        } else {
            Err(format!("{relative} resolves outside the worktree"))
        }
    }
}

fn spec(name: &str, description: &str, parameters: Value) -> ToolSpec {
    ToolSpec {
        name: name.to_owned(),
        description: description.to_owned(),
        parameters,
    }
}

/// The path as `/`-separated normal components, or why it is not a file inside the worktree.
fn confined(path: &str) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) if part == OsStr::new(".git") => {
                return Err(format!("{path} is inside .git"));
            }
            Component::Normal(part) => parts.push(
                part.to_str()
                    .ok_or_else(|| format!("{path} is not UTF-8"))?
                    .to_owned(),
            ),
            Component::CurDir => {}
            _ => {
                return Err(format!(
                    "{path} must be relative and stay inside the worktree"
                ));
            }
        }
    }
    if parts.is_empty() {
        return Err(format!("{path:?} names no file"));
    }
    Ok(parts.join("/"))
}

fn shell(command: &str) -> Command {
    if cfg!(windows) {
        let mut shell = Command::new("cmd");
        shell.arg("/C").arg(command);
        shell
    } else {
        let mut shell = Command::new("sh");
        shell.arg("-c").arg(command);
        shell
    }
}

/// The last `max` bytes of `text`, moved forward to a character boundary.
fn tail(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut start = text.len() - max;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    &text[start..]
}

/// Runs git in `dir`. A failure returns git's own words, from stderr or else stdout.
async fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .await
        .map_err(|err| format!("git did not start: {err}"))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if !stderr.is_empty() {
        return Err(stderr);
    }
    Err(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
