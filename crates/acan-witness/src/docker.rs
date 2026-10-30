//! The `docker create` argument list and the mounts of one probe sequence container.
//!
//! Each field of the [`ContainerPolicy`] is applied here or the run is refused: a runner that
//! cannot apply a field never runs with less isolation. The list sets one environment variable,
//! `HOME`. The app's port reaches the app through the driver, and nothing is mounted but the
//! worktree, the protected paths from the snapshot, the harness and `.git` when the policy keeps
//! it read-only.

use std::num::NonZeroU32;
use std::path::Path;

use acan_intent::{ContainerNetwork, ContainerPolicy, MissingProtected};

/// Where the harness directory is mounted in the container. Always read-only.
pub(crate) const HARNESS_TARGET: &str = "/harness";

/// The label key every container this runner starts carries. The value is the run id, so
/// cleanup and tests can find the containers of one run.
pub(crate) const LABEL_KEY: &str = "acan.witness";

/// The host paths a container's mounts are built from.
pub(crate) struct MountSources<'a> {
    /// The worktree, mounted read-write at the policy's working directory.
    pub worktree: &'a Path,
    /// The protected-head snapshot the protected paths are mounted from, read-only.
    pub snapshot: &'a Path,
    /// The directory holding the harness, mounted read-only at [`HARNESS_TARGET`].
    pub harness: &'a Path,
    /// An empty directory, mounted over a missing protected path when the policy masks it.
    pub empty: &'a Path,
}

/// Builds the `--mount` values of a container.
///
/// # Errors
///
/// Returns the refusal reason when a path cannot be written as a mount value, or when the policy
/// refuses a protected path the snapshot does not hold.
pub(crate) fn mount_values(
    policy: &ContainerPolicy,
    sources: &MountSources<'_>,
) -> Result<Vec<String>, String> {
    let workdir = policy.mounts.workdir.as_str();
    let mut mounts = vec![bind(sources.worktree, workdir, false)?];
    let git = sources.worktree.join(".git");
    if policy.mounts.readonly_git && git.exists() {
        mounts.push(bind(&git, &format!("{workdir}/.git"), true)?);
    }
    for protected in &policy.mounts.protected {
        let target = format!("{workdir}/{}", protected.as_str());
        let source = sources.snapshot.join(protected.as_str());
        if source.exists() {
            mounts.push(bind(&source, &target, true)?);
            continue;
        }
        match policy.mounts.missing_protected {
            MissingProtected::Masked => mounts.push(bind(sources.empty, &target, true)?),
            MissingProtected::Refused => {
                return Err(format!(
                    "protected path {} is missing from the snapshot",
                    protected.as_str()
                ));
            }
        }
    }
    mounts.push(bind(sources.harness, HARNESS_TARGET, true)?);
    Ok(mounts)
}

/// The `--mount` value for one bind mount. A comma or quote in a path would split the value,
/// so such a path is refused rather than quoted.
fn bind(source: &Path, target: &str, readonly: bool) -> Result<String, String> {
    let Some(source_text) = source.to_str() else {
        return Err(format!("mount source is not UTF-8: {}", source.display()));
    };
    if [source_text, target]
        .iter()
        .any(|text| text.contains([',', '"']))
    {
        return Err(format!(
            "mount path cannot be passed to docker: {source_text}"
        ));
    }
    let mut value = format!("type=bind,source={source_text},target={target}");
    if readonly {
        value.push_str(",readonly");
    }
    Ok(value)
}

/// The `docker create` arguments of one probe sequence container.
///
/// The container is created first and started separately, so its id is known before it runs and
/// it can always be removed by id. Once started it runs `bun /harness/driver.ts` with stdin open:
/// the host writes one JSON line per operation. `--rm` removes the container when it exits.
///
/// # Errors
///
/// Returns the refusal reason when the policy asks for something this runner does not apply:
/// a network other than `none`, or an image that is not pinned by digest.
pub(crate) fn create_args(
    name: &str,
    run_id: &str,
    policy: &ContainerPolicy,
    mounts: &[String],
) -> Result<Vec<String>, String> {
    if !matches!(policy.network, ContainerNetwork::None {}) {
        return Err(
            "the container network must be none; loopback publishing is not applied".to_owned(),
        );
    }
    let Some(digest) = policy.image.digest.as_ref() else {
        return Err("the image must be pinned by digest".to_owned());
    };
    let image = format!("{}@{digest}", policy.image.name.as_str());
    let memory = policy.memory_bytes.to_string();

    let mut args: Vec<String> = Vec::new();
    args.extend(
        [
            "create",
            "--interactive",
            "--rm",
            "--pull",
            "never",
            "--name",
        ]
        .map(str::to_owned),
    );
    args.push(name.to_owned());
    args.push("--label".to_owned());
    args.push(format!("{LABEL_KEY}={run_id}"));
    args.extend(
        [
            "--network",
            "none",
            "--read-only",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "--user",
        ]
        .map(str::to_owned),
    );
    args.push(policy.user.to_string());
    args.extend(["--cpus".to_owned(), cpus(policy.cpus_milli)]);
    args.extend(["--memory".to_owned(), memory.clone()]);
    args.extend(["--memory-swap".to_owned(), memory]);
    args.extend(["--pids-limit".to_owned(), policy.pids.to_string()]);
    args.extend([
        "--tmpfs".to_owned(),
        format!("/tmp:rw,nosuid,nodev,size={}", policy.tmp_bytes),
    ]);
    args.extend(["--env".to_owned(), "HOME=/tmp".to_owned()]);
    args.extend([
        "--workdir".to_owned(),
        policy.mounts.workdir.as_str().to_owned(),
    ]);
    for mount in mounts {
        args.push("--mount".to_owned());
        args.push(mount.clone());
    }
    args.extend(["--entrypoint", "bun"].map(str::to_owned));
    args.push(image);
    args.push(format!("{HARNESS_TARGET}/driver.ts"));
    Ok(args)
}

/// The `--cpus` value for a cap in thousandths of a core.
fn cpus(milli: NonZeroU32) -> String {
    format!("{:.3}", f64::from(milli.get()) / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pinned_policy(missing: &str) -> ContainerPolicy {
        serde_json::from_value(serde_json::json!({
            "image": {
                "name": "oven/bun:1.4.4",
                "digest": "sha256:1d22548f177f4ff38e0c15baa3c2c46c6c2060ba2d1bc573bb4720b407548f1d"
            },
            "user": "1000:1000",
            "cpus_milli": 2000,
            "memory_bytes": 1073741824u64,
            "pids": 256,
            "timeout_ms": 60000,
            "tmp_bytes": 268435456u64,
            "network": { "kind": "none" },
            "root": "read_only",
            "privileges": "dropped",
            "mounts": {
                "workdir": "/work",
                "protected": [".acan"],
                "missing_protected": missing
            }
        }))
        .expect("the test policy decodes")
    }

    #[test]
    fn create_args_are_exact_for_the_pinned_policy() {
        let policy = pinned_policy("refused");
        let mounts = vec![
            "type=bind,source=W,target=/work".to_owned(),
            "type=bind,source=W\\.git,target=/work/.git,readonly".to_owned(),
            "type=bind,source=S\\.acan,target=/work/.acan,readonly".to_owned(),
            "type=bind,source=H,target=/harness,readonly".to_owned(),
        ];
        let args = create_args("acan-witness-R", "R", &policy, &mounts).expect("args build");
        let expected = [
            "create",
            "--interactive",
            "--rm",
            "--pull",
            "never",
            "--name",
            "acan-witness-R",
            "--label",
            "acan.witness=R",
            "--network",
            "none",
            "--read-only",
            "--cap-drop",
            "ALL",
            "--security-opt",
            "no-new-privileges",
            "--user",
            "1000:1000",
            "--cpus",
            "2.000",
            "--memory",
            "1073741824",
            "--memory-swap",
            "1073741824",
            "--pids-limit",
            "256",
            "--tmpfs",
            "/tmp:rw,nosuid,nodev,size=268435456",
            "--env",
            "HOME=/tmp",
            "--workdir",
            "/work",
            "--mount",
            "type=bind,source=W,target=/work",
            "--mount",
            "type=bind,source=W\\.git,target=/work/.git,readonly",
            "--mount",
            "type=bind,source=S\\.acan,target=/work/.acan,readonly",
            "--mount",
            "type=bind,source=H,target=/harness,readonly",
            "--entrypoint",
            "bun",
            "oven/bun:1.4.4@sha256:1d22548f177f4ff38e0c15baa3c2c46c6c2060ba2d1bc573bb4720b407548f1d",
            "/harness/driver.ts",
        ];
        assert_eq!(args, expected, "the docker argument list changed");
    }

    #[test]
    fn only_home_is_set_in_the_environment() {
        let policy = pinned_policy("refused");
        let args = create_args("n", "r", &policy, &[]).expect("args build");
        let env_values: Vec<&String> = args
            .windows(2)
            .filter(|pair| pair[0] == "--env")
            .map(|pair| &pair[1])
            .collect();
        assert_eq!(env_values, vec!["HOME=/tmp"]);
        assert!(
            !args
                .iter()
                .any(|arg| arg.contains(".env") || arg.contains("KEY")),
            "no env file or key may enter the argument list: {args:?}"
        );
    }

    #[test]
    fn loopback_publishing_is_refused() {
        let mut policy = pinned_policy("refused");
        policy.network = ContainerNetwork::LoopbackPublished {
            port: std::num::NonZeroU16::new(3000).expect("nonzero"),
        };
        let refusal = create_args("n", "r", &policy, &[]).expect_err("publishing is refused");
        assert!(refusal.contains("network must be none"), "{refusal}");
    }

    #[test]
    fn an_undigested_image_is_refused() {
        let mut policy = pinned_policy("refused");
        policy.image.digest = None;
        let refusal =
            create_args("n", "r", &policy, &[]).expect_err("an unpinned image is refused");
        assert!(refusal.contains("pinned by digest"), "{refusal}");
    }

    #[test]
    fn a_missing_protected_path_is_refused_by_policy() {
        let dir = tempfile::tempdir().expect("temp dir");
        let worktree = dir.path().join("worktree");
        let snapshot = dir.path().join("snapshot");
        let harness = dir.path().join("harness");
        let empty = dir.path().join("empty");
        for path in [&worktree, &snapshot, &harness, &empty] {
            std::fs::create_dir_all(path).expect("dir");
        }
        let sources = MountSources {
            worktree: &worktree,
            snapshot: &snapshot,
            harness: &harness,
            empty: &empty,
        };
        let refusal = mount_values(&pinned_policy("refused"), &sources)
            .expect_err("the snapshot lacks .acan");
        assert!(refusal.contains(".acan"), "{refusal}");
    }

    #[test]
    fn a_missing_protected_path_is_masked_by_an_empty_mount() {
        let dir = tempfile::tempdir().expect("temp dir");
        let worktree = dir.path().join("worktree");
        let snapshot = dir.path().join("snapshot");
        let harness = dir.path().join("harness");
        let empty = dir.path().join("empty");
        for path in [&worktree, &snapshot, &harness, &empty] {
            std::fs::create_dir_all(path).expect("dir");
        }
        let sources = MountSources {
            worktree: &worktree,
            snapshot: &snapshot,
            harness: &harness,
            empty: &empty,
        };
        let mounts = mount_values(&pinned_policy("masked"), &sources).expect("masked mounts");
        let masked = format!(
            "type=bind,source={},target=/work/.acan,readonly",
            empty.display()
        );
        assert!(mounts.contains(&masked), "{mounts:?}");
    }

    #[test]
    fn a_comma_in_a_mount_path_is_refused() {
        let dir = tempfile::tempdir().expect("temp dir");
        let odd = dir.path().join("a,b");
        let refusal = bind(&odd, "/work", false).expect_err("comma refused");
        assert!(refusal.contains("cannot be passed"), "{refusal}");
    }
}
