//! The worker's local tools. Every path stays inside the task's worktree, a write is allowed
//! only to a file the task declared, and no tool writes or commits a protected path, declared
//! or not. The worker decides when a write may run (see `worker`).
//!
//! The path checks bind these tools and nothing else; they are not a sandbox. Code the test
//! command runs is held back only by the container [`Worktree::run_tests`] starts, and when no
//! container runtime answers, the test command does not run at all.

use std::fmt::Display;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use acan_intent::{Isolation, RepoPath};
use acan_llm::ToolSpec;
use serde_json::{Value, json};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

/// The most bytes a read returns. Longer files are cut and marked.
const MAX_READ_BYTES: usize = 32 * 1024;
/// The most entries a directory listing returns.
const MAX_LISTING: usize = 200;

/// Paths no tool writes or commits, whatever the task declares.
const ALWAYS_PROTECTED: &[&str] = &[".acan"];

/// How every refusal for a missing or failed container runtime begins.
const ISOLATION_UNAVAILABLE: &str = "isolation unavailable";
const DEFAULT_DOCKER: &str = "docker";
const DEFAULT_IMAGE: &str = "oven/bun:1";
/// Where the worktree is mounted inside the container.
const CONTAINER_WORKDIR: &str = "/work";
const CONTAINER_USER: &str = "1000:1000";
const CONTAINER_CPUS: &str = "2";
const CONTAINER_MEMORY: &str = "1g";
const CONTAINER_PIDS: &str = "256";
/// The root filesystem is read-only, so the only scratch space is a bounded tmpfs.
const CONTAINER_TMP: &str = "/tmp:rw,nosuid,nodev,size=256m";
const CONTAINER_HOME: &str = "HOME=/tmp";
const TEST_TIMEOUT: Duration = Duration::from_secs(120);
const DOCKER_ANSWER_TIMEOUT: Duration = Duration::from_secs(15);
/// Bytes kept from the end of the test command's stdout and of its stderr.
const OUTPUT_TAIL_BYTES: usize = 4 * 1024;

/// One task's worktree, with the files it declared, the paths it may never change, and the
/// test command it may run.
#[derive(Debug)]
pub struct Worktree {
    root: PathBuf,
    declared: Vec<String>,
    protected: Vec<String>,
    test_command: Option<String>,
    container: Container,
}

/// The container the test command runs in.
#[derive(Debug)]
struct Container {
    docker: PathBuf,
    image: String,
    isolation: Isolation,
}

/// A path inside the worktree: as joined from the request, and as it really is on disk.
struct Resolved {
    path: PathBuf,
    /// The `/`-separated path from the worktree root after every link, case difference and
    /// short name is resolved.
    real: String,
}

impl Worktree {
    /// The worktree at `root` for a task that declared `declared`. The declared paths are
    /// normalized the way [`normalize_path`] does; a path that cannot be normalized is dropped,
    /// so it can never be written. The worker only passes paths it has already normalized.
    ///
    /// `.acan/` is protected from the start. The test command runs with `docker` from `PATH`
    /// in the `oven/bun:1` image.
    #[must_use]
    pub fn new(root: PathBuf, declared: &[String], test_command: Option<String>) -> Self {
        let declared = declared
            .iter()
            .filter_map(|path| normalize_path(path).ok())
            .collect();
        Self {
            root,
            declared,
            protected: ALWAYS_PROTECTED
                .iter()
                .map(|&path| path.to_owned())
                .collect(),
            test_command,
            container: Container {
                docker: PathBuf::from(DEFAULT_DOCKER),
                image: DEFAULT_IMAGE.to_owned(),
                isolation: Isolation::default(),
            },
        }
    }

    /// Protects `paths` as well as `.acan/`. A protected path covers itself and everything
    /// under it. No tool writes, restores or commits it, even when the task declares it, and
    /// the test command's container mounts it read-only.
    #[must_use]
    pub fn with_protected(mut self, paths: &[RepoPath]) -> Self {
        self.protected.extend(paths.iter().map(|path| {
            normalize_path(path.as_str()).unwrap_or_else(|_| path.as_str().to_owned())
        }));
        self
    }

    /// Runs the test command's container through the docker CLI at `docker`.
    #[must_use]
    pub fn with_docker(mut self, docker: impl Into<PathBuf>) -> Self {
        self.container.docker = docker.into();
        self
    }

    /// Runs the test command in `image`.
    #[must_use]
    pub fn with_image(mut self, image: impl Into<String>) -> Self {
        self.container.image = image.into();
        self
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
                "Write a file you declared. Protected paths such as .acan/ are never written. The write waits until the origin clears the task to build.",
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
                "Run the test command the worker was started with, in a container over the worktree with no network and protected paths read-only.",
                json!({ "type": "object", "properties": {} }),
            ),
            spec(
                "commit",
                "Commit every change in the worktree. Only declared files may change, and protected paths may not change at all.",
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
        let relative = normalize_path(path)?;
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
            self.inside(&normalize_path(path)?)?
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

    /// Writes a declared file, creating its directories. The write does not go through a link,
    /// so it lands on the declared path and nowhere else, and never on a protected path.
    ///
    /// # Errors
    ///
    /// Returns the reason when the path is protected under any spelling, the file is not
    /// declared, its path runs through a link, or it cannot be written.
    pub fn write_file(&self, path: &str, content: &str) -> Result<String, String> {
        let relative = normalize_path(path)?;
        self.refuse_protected(&relative)?;
        if !self.declared.contains(&relative) {
            return Err(format!(
                "{relative} is not declared; declared files are: {}",
                self.declared.join(", ")
            ));
        }
        self.no_links(&relative)?;
        let resolved = self.resolve(&relative)?;
        // A Windows short name such as `ACAN~1` names a protected directory the plain spelling
        // check cannot see; the resolved path shows it.
        self.refuse_protected(&resolved.real)?;
        if let Some(parent) = resolved.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("cannot create the directory for {relative}: {err}"))?;
        }
        std::fs::write(&resolved.path, content)
            .map_err(|err| format!("cannot write {relative}: {err}"))?;
        Ok(format!("wrote {} bytes to {relative}", content.len()))
    }

    /// Runs the configured test command in a fresh container over the worktree and reports
    /// its outcome. The command never runs on the host: when the container runtime is missing
    /// or does not answer, the reason begins with `isolation unavailable`.
    ///
    /// # Errors
    ///
    /// Returns the reason when no test command is configured, the container cannot run, or
    /// the command timed out.
    pub async fn run_tests(&self) -> Result<String, String> {
        let Some(command) = &self.test_command else {
            return Err(no_test_command());
        };
        let name = container_name();
        let args = self.container_args(&name, command).map_err(unavailable)?;
        self.container.answers().await?;
        self.container.run(&name, &args).await
    }

    /// The `docker` arguments that run the test command in a container named `name`: a fresh
    /// container removed after the run, as a non-root user, on a read-only root filesystem
    /// with a bounded `/tmp`, no network, no capabilities, CPU, memory and process limits, and
    /// `HOME` as its only environment variable. The worktree is mounted read-write, and `.git`
    /// and every protected path that exists are mounted read-only on top of it.
    ///
    /// # Errors
    ///
    /// Returns the reason when no test command is configured, the image name is not usable,
    /// or the worktree or a protected path cannot be mounted (a protected path that is a link
    /// is refused).
    pub fn test_container_args(&self, name: &str) -> Result<Vec<String>, String> {
        let command = self.test_command.as_deref().ok_or_else(no_test_command)?;
        self.container_args(name, command)
    }

    fn container_args(&self, name: &str, command: &str) -> Result<Vec<String>, String> {
        let image = &self.container.image;
        if image.is_empty() || image.starts_with('-') {
            return Err(format!("{image:?} is not an image name"));
        }
        let root = std::path::absolute(&self.root)
            .map_err(|err| format!("the worktree cannot be resolved: {err}"))?;
        let mut args = vec!["run".to_owned()];
        match self.container.isolation {
            Isolation::FreshInstance => args.push("--rm".to_owned()),
        }
        args.extend(
            [
                "--name",
                name,
                "--network",
                "none",
                "--read-only",
                "--user",
                CONTAINER_USER,
                "--cap-drop",
                "ALL",
                "--security-opt",
                "no-new-privileges",
                "--cpus",
                CONTAINER_CPUS,
                "--memory",
                CONTAINER_MEMORY,
                "--memory-swap",
                CONTAINER_MEMORY,
                "--pids-limit",
                CONTAINER_PIDS,
                "--tmpfs",
                CONTAINER_TMP,
                "--env",
                CONTAINER_HOME,
                "--mount",
            ]
            .map(str::to_owned),
        );
        args.push(bind(&root, CONTAINER_WORKDIR)?);
        let mut mounted: Vec<String> = Vec::new();
        // `.git` is read-only too: hooks written there would run on the host at the next commit.
        for relative in std::iter::once(".git").chain(self.protected.iter().map(String::as_str)) {
            let Some(real) = self.existing_real(relative)? else {
                continue;
            };
            if mounted.contains(&real) {
                continue;
            }
            let source = real
                .split('/')
                .fold(root.clone(), |path, part| path.join(part));
            args.push("--mount".to_owned());
            args.push(format!(
                "{},readonly",
                bind(&source, &format!("{CONTAINER_WORKDIR}/{real}"))?
            ));
            mounted.push(real);
        }
        args.extend(
            [
                "--workdir",
                CONTAINER_WORKDIR,
                "--entrypoint",
                "sh",
                image,
                "-c",
                command,
            ]
            .map(str::to_owned),
        );
        Ok(args)
    }

    /// The resolved form of `relative` when it exists, refusing one that runs through a link,
    /// because a read-only mount of a link would expose whatever it points to.
    fn existing_real(&self, relative: &str) -> Result<Option<String>, String> {
        match self.root.join(relative).symlink_metadata() {
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(format!("cannot inspect {relative}: {err}")),
        }
        self.no_links(relative)?;
        self.resolve(relative).map(|resolved| Some(resolved.real))
    }

    /// Stages every change, refuses it when a protected path or a file outside the declaration
    /// changed, and commits. Returns the new commit id.
    ///
    /// # Errors
    ///
    /// Returns the reason when git fails, when nothing changed, or when a protected path or a
    /// file outside the declaration changed, deleted or renamed included.
    pub async fn commit(&self, message: &str) -> Result<String, String> {
        git(&self.root, &["add", "-A"]).await?;
        // Without `--no-renames`, a protected file renamed to a declared path lists only the
        // declared side, and the deletion of the protected file would go unseen.
        let staged = git(
            &self.root,
            &["diff", "--cached", "--name-only", "--no-renames", "-z"],
        )
        .await?;
        let changed: Vec<&str> = staged.split('\0').filter(|file| !file.is_empty()).collect();
        let protected: Vec<&str> = changed
            .iter()
            .copied()
            .filter(|file| self.is_protected(file))
            .collect();
        let outside: Vec<&str> = changed
            .iter()
            .copied()
            .filter(|file| !self.is_protected(file) && !self.declared.iter().any(|d| d == file))
            .collect();
        let mut reasons = Vec::new();
        if !protected.is_empty() {
            reasons.push(format!("{} is protected", protected.join(", ")));
        }
        if !outside.is_empty() {
            reasons.push(format!(
                "{} changed outside the declared files",
                outside.join(", ")
            ));
        }
        if !reasons.is_empty() {
            // The refused change stays in the worktree, unstaged, for the worker to see.
            let _ = git(&self.root, &["reset", "-q"]).await;
            return Err(format!("commit refused: {}", reasons.join("; ")));
        }
        git(&self.root, &["commit", "-q", "-m", message]).await?;
        Ok(git(&self.root, &["rev-parse", "HEAD"])
            .await?
            .trim()
            .to_owned())
    }

    /// Sets the files the task declares now. Writes to any other file are refused from here on.
    pub fn declare(&mut self, declared: &[String]) {
        self.declared = declared
            .iter()
            .filter_map(|path| normalize_path(path).ok())
            .collect();
    }

    /// Restores files a narrowing took from the declaration. A tracked file goes back to its
    /// committed version. A file the task never committed is removed, so a later commit cannot
    /// carry the change. A protected path is left alone: no tool wrote it, so there is nothing
    /// to undo, and removing it would delete a protected input.
    ///
    /// # Errors
    ///
    /// Returns the reason when git cannot restore a file or a file cannot be removed.
    pub async fn restore(&self, files: &[String]) -> Result<(), String> {
        for relative in files {
            if self.is_protected(relative) {
                continue;
            }
            let path = self.inside(relative)?;
            // `ls-files` prints the path when git tracks it and nothing when it does not. A git
            // failure is returned as an error, so a file is never removed on a guess.
            let listed = git(
                &self.root,
                &["--literal-pathspecs", "ls-files", "--", relative],
            )
            .await?;
            if listed.trim().is_empty() {
                if path.exists() {
                    std::fs::remove_file(&path)
                        .map_err(|err| format!("cannot remove {relative}: {err}"))?;
                }
            } else {
                git(
                    &self.root,
                    &["--literal-pathspecs", "checkout", "-q", "--", relative],
                )
                .await?;
            }
        }
        Ok(())
    }

    /// Whether `relative` is a protected path or lies under one. Case is ignored, because
    /// Windows and macOS name one file by every case of its name.
    fn is_protected(&self, relative: &str) -> bool {
        let path = relative.to_lowercase();
        self.protected.iter().any(|protected| {
            let protected = protected.to_lowercase();
            path.strip_prefix(&protected)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
        })
    }

    fn refuse_protected(&self, relative: &str) -> Result<(), String> {
        if self.is_protected(relative) {
            return Err(format!(
                "{relative} is protected; no tool writes or commits it, even when declared"
            ));
        }
        Ok(())
    }

    /// The path inside the worktree. Every existing link on the way is resolved, so a symlink
    /// that points outside the worktree is refused, and so is a link that cannot be resolved.
    fn inside(&self, relative: &str) -> Result<PathBuf, String> {
        self.resolve(relative).map(|resolved| resolved.path)
    }

    fn resolve(&self, relative: &str) -> Result<Resolved, String> {
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
        let below_root = real
            .strip_prefix(&root)
            .map_err(|_| format!("{relative} resolves outside the worktree"))?;
        let missing = joined
            .strip_prefix(probe)
            .map_err(|_| format!("cannot resolve {relative}"))?;
        let mut parts = Vec::new();
        for component in below_root.components().chain(missing.components()) {
            match component {
                Component::Normal(part) => {
                    parts.push(part.to_str().ok_or_else(|| {
                        format!("{relative} resolves to a name that is not UTF-8")
                    })?);
                }
                _ => return Err(format!("cannot resolve {relative}")),
            }
        }
        Ok(Resolved {
            real: parts.join("/"),
            path: joined,
        })
    }

    /// Refuses a write whose path runs through a link. The declaration names one file, and a link
    /// can lead to any other file, so no write follows one.
    fn no_links(&self, relative: &str) -> Result<(), String> {
        let mut current = self.root.clone();
        for part in relative.split('/') {
            current.push(part);
            match std::fs::symlink_metadata(&current) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    return Err(format!(
                        "{relative} runs through a link, and writes do not follow links"
                    ));
                }
                Ok(_) => {}
                // Nothing exists past a missing component, so the rest is created as plain files
                // and directories.
                Err(_) => break,
            }
        }
        Ok(())
    }
}

impl Container {
    /// Checks that the docker CLI starts and its daemon answers.
    async fn answers(&self) -> Result<(), String> {
        let mut version = Command::new(&self.docker);
        version
            .args(["version", "--format", "{{.Server.Version}}"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let output = match tokio::time::timeout(DOCKER_ANSWER_TIMEOUT, version.output()).await {
            Err(_) => return Err(unavailable("the docker daemon did not answer")),
            Ok(Err(err)) => {
                return Err(unavailable(format!(
                    "{} did not start: {err}",
                    self.docker.display()
                )));
            }
            Ok(Ok(output)) => output,
        };
        if output.status.success() {
            return Ok(());
        }
        let said = String::from_utf8_lossy(&output.stderr);
        Err(unavailable(format!(
            "the docker daemon is not reachable: {}",
            said.trim()
        )))
    }

    async fn run(&self, name: &str, args: &[String]) -> Result<String, String> {
        let mut child = Command::new(&self.docker)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|err| {
                unavailable(format!("{} did not start: {err}", self.docker.display()))
            })?;
        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            return Err("the docker output is not piped".to_owned());
        };
        let stdout = tokio::spawn(read_tail(stdout));
        let stderr = tokio::spawn(read_tail(stderr));
        let status = match tokio::time::timeout(TEST_TIMEOUT, child.wait()).await {
            Err(_) => {
                // Killing the docker CLI leaves its container running, so the container is
                // killed by name.
                self.kill(name).await;
                let _ = child.kill().await;
                return Err(format!(
                    "the test command timed out after {}s; its container was killed",
                    TEST_TIMEOUT.as_secs()
                ));
            }
            Ok(Err(err)) => return Err(format!("could not wait on docker: {err}")),
            Ok(Ok(status)) => status,
        };
        let output = shown(
            &stdout.await.unwrap_or_default(),
            &stderr.await.unwrap_or_default(),
        );
        match status.code() {
            Some(0) => Ok("pass".to_owned()),
            // 125 is docker's own failure: the container never ran the command.
            Some(125) => Err(unavailable(format!(
                "docker could not start the container{output}"
            ))),
            Some(code) => Ok(format!("fail: exit code {code}{output}")),
            None => Ok(format!("fail: killed by a signal{output}")),
        }
    }

    async fn kill(&self, name: &str) {
        let mut kill = Command::new(&self.docker);
        kill.args(["kill", name])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let _ = tokio::time::timeout(DOCKER_ANSWER_TIMEOUT, kill.status()).await;
    }
}

/// A `--mount` value binding `source` on the host to `target` in the container.
fn bind(source: &Path, target: &str) -> Result<String, String> {
    let source = source
        .to_str()
        .ok_or_else(|| format!("{} is not UTF-8", source.display()))?;
    // docker reads `--mount` as comma-separated fields, so a comma or quote would end a field.
    for path in [source, target] {
        if path.contains([',', '"', '\n', '\r']) {
            return Err(format!("{path:?} cannot be mounted"));
        }
    }
    Ok(format!("type=bind,source={source},target={target}"))
}

/// A container name no other run of this or another worker shares.
fn container_name() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    format!(
        "acan-test-{}-{nanos}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

fn no_test_command() -> String {
    "no test command is configured for this worker".to_owned()
}

fn unavailable(detail: impl Display) -> String {
    format!("{ISOLATION_UNAVAILABLE}: {detail}")
}

/// The ends of the command's output, each on its own lines, or nothing when it printed none.
fn shown(stdout: &[u8], stderr: &[u8]) -> String {
    [("stdout", stdout), ("stderr", stderr)]
        .into_iter()
        .filter(|(_, bytes)| !bytes.is_empty())
        .map(|(label, bytes)| {
            format!(
                "\n{label} (last {} bytes):\n{}",
                bytes.len(),
                String::from_utf8_lossy(bytes)
            )
        })
        .collect()
}

/// The last [`OUTPUT_TAIL_BYTES`] bytes `reader` yields.
async fn read_tail<R: AsyncRead + Unpin>(mut reader: R) -> Vec<u8> {
    let mut tail = Vec::new();
    let mut buf = [0u8; 8192];
    while let Ok(read @ 1..) = reader.read(&mut buf).await {
        tail.extend_from_slice(&buf[..read]);
        if tail.len() > 2 * OUTPUT_TAIL_BYTES {
            tail.drain(..tail.len() - OUTPUT_TAIL_BYTES);
        }
    }
    let excess = tail.len().saturating_sub(OUTPUT_TAIL_BYTES);
    tail.drain(..excess);
    tail
}

fn spec(name: &str, description: &str, parameters: Value) -> ToolSpec {
    ToolSpec {
        name: name.to_owned(),
        description: description.to_owned(),
        parameters,
    }
}
/// The repository-relative form of `path` that the origin keeps: `/`-separated normal components.
/// A `.` is dropped, so `./model/store.ts` and `model/store.ts` are one path to the origin and to
/// the worktree alike.
///
/// # Errors
///
/// Returns the reason when the path is absolute, climbs out with `..`, names nothing, is inside
/// `.git`, or has a name the origin does not keep as a path.
pub fn normalize_path(path: &str) -> Result<String, String> {
    let mut parts = Vec::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => {
                let part = part
                    .to_str()
                    .ok_or_else(|| format!("{path:?} is not UTF-8"))?;
                if part.eq_ignore_ascii_case(".git") {
                    return Err(format!("{path} is inside .git"));
                }
                // The origin keeps neither `\` nor `:`, and Windows drops a trailing dot or space,
                // so such a name would name a different file on one side than the other.
                if part.contains(['\\', ':']) || part.ends_with(['.', ' ']) {
                    return Err(format!(
                        "{path:?} has a name the origin does not keep as a path"
                    ));
                }
                parts.push(part);
            }
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

/// Runs git in `dir`. A failure returns git's own words, from stderr or else stdout.
pub(crate) async fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
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
