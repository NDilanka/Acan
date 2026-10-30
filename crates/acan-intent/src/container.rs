//! The container policy of every isolated run: the candidate app, every probe sequence, and the
//! worker's test command.
//!
//! Policy rules, binding on every runner:
//! - The command runs as a non-root user on a read-only root filesystem with a bounded `/tmp`,
//!   with no capabilities and no privilege gain, under the CPU, memory, process and time caps
//!   below. The container is killed and removed when it ends or times out.
//! - The worktree is mounted read-write at the working directory. Every protected path is
//!   mounted read-only on top of it from a snapshot of the protected head, never from the
//!   worktree. A protected path the snapshot does not have is still covered as
//!   [`MissingProtected`] says; it is never left writable.
//! - The container reaches only what [`ContainerNetwork`] allows. No provider key, owner token
//!   or host secret enters it.
//! - A runner that cannot apply every field refuses the run. It never runs with less isolation.
//!
//! The policy's [`SandboxPolicyHash`] is part of the [`crate::GateContext`], so changing any
//! field voids a clearance obtained under the old policy.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::num::{NonZeroU16, NonZeroU32, NonZeroU64};
use std::str::FromStr;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::{CanonicalError, ContentHash, ImageDigest, SandboxPolicyHash};
use crate::types::{ContainerPath, ImageName, InvalidValue, RepoPath};

/// The image a run starts from.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerImage {
    /// The name and tag, such as `oven/bun:1`.
    pub name: ImageName,
    /// The registry digest. When present the runner starts exactly this content and refuses an
    /// image whose digest differs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<ImageDigest>,
}

/// The numeric user and group a run executes as, written `uid:gid`. Neither is ever 0, so a run
/// is never root and never in the root group.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ContainerUser {
    /// The user id.
    pub uid: NonZeroU32,
    /// The group id.
    pub gid: NonZeroU32,
}

impl fmt::Display for ContainerUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.uid, self.gid)
    }
}

impl FromStr for ContainerUser {
    type Err = InvalidValue;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        // Digits only, no sign and no leading zero, so the text re-encodes to itself.
        let id = |part: &str| {
            let canonical = part.starts_with(|c: char| matches!(c, '1'..='9'))
                && part.bytes().all(|b| b.is_ascii_digit());
            canonical.then(|| part.parse::<NonZeroU32>().ok()).flatten()
        };
        raw.split_once(':')
            .and_then(|(uid, gid)| {
                Some(Self {
                    uid: id(uid)?,
                    gid: id(gid)?,
                })
            })
            .ok_or_else(|| InvalidValue::Malformed {
                what: "container user",
                raw: raw.to_owned(),
            })
    }
}

impl Serialize for ContainerUser {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContainerUser {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl JsonSchema for ContainerUser {
    fn schema_name() -> Cow<'static, str> {
        "ContainerUser".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^[1-9][0-9]*:[1-9][0-9]*$",
            "description": "uid:gid, neither of them 0"
        })
    }
}

/// What a run's container may reach. Closed: there is no host or bridge network.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContainerNetwork {
    /// No network at all. The worker's test command runs this way.
    None {},
    /// Only the app's port, published on the host's loopback interface for the runner. The
    /// container has no other route out.
    LoopbackPublished {
        /// The port the app listens on inside the container.
        port: NonZeroU16,
    },
}

/// The container's root filesystem. Only `/tmp` and the worktree are writable.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RootFilesystem {
    /// Mounted read-only.
    ReadOnly,
}

/// The privileges the run keeps.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Privileges {
    /// Every capability dropped, and no privilege gain through setuid or file capabilities.
    Dropped,
}

/// How a protected path the protected-head snapshot does not have is covered.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MissingProtected {
    /// An empty read-only mount covers the path, so the run can neither read a copy the
    /// candidate added nor create one.
    Masked,
    /// The run is refused.
    Refused,
}

/// What is mounted where.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MountPolicy {
    /// Where the worktree is mounted read-write. Also the working directory.
    pub workdir: ContainerPath,
    /// The worktree paths mounted read-only from the protected-head snapshot over the worktree:
    /// `.acan` and the protected tests.
    pub protected: BTreeSet<RepoPath>,
    /// How a protected path missing from the snapshot is covered.
    pub missing_protected: MissingProtected,
    /// Whether `.git` is mounted read-only over the worktree. Hooks written there would run on
    /// the host at the next commit, so a policy keeps it read-only unless it says otherwise.
    #[serde(default = "readonly_git_by_default")]
    pub readonly_git: bool,
}

/// The default for [`MountPolicy::readonly_git`]: `.git` is mounted read-only.
const fn readonly_git_by_default() -> bool {
    true
}

/// The container policy of every isolated run. Its hash is the [`SandboxPolicyHash`].
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerPolicy {
    /// The image.
    pub image: ContainerImage,
    /// Who the command runs as.
    pub user: ContainerUser,
    /// CPU cap, in thousandths of a core.
    pub cpus_milli: NonZeroU32,
    /// Memory cap. Swap is not allowed beyond it.
    pub memory_bytes: NonZeroU64,
    /// Process cap.
    pub pids: NonZeroU32,
    /// Wall-clock cap from container start. At it the runner kills and removes the container.
    pub timeout_ms: NonZeroU64,
    /// Size of the writable `/tmp`, which is also `HOME`.
    pub tmp_bytes: NonZeroU64,
    /// What the container may reach.
    pub network: ContainerNetwork,
    /// The root filesystem.
    pub root: RootFilesystem,
    /// The privileges kept.
    pub privileges: Privileges,
    /// The mounts.
    pub mounts: MountPolicy,
}

impl ContainerPolicy {
    /// The policy's identity, carried in [`crate::GateContext::sandbox_policy_hash`].
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn sandbox_policy_hash(&self) -> Result<SandboxPolicyHash, CanonicalError> {
        ContentHash::of(self).map(SandboxPolicyHash::from)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn fixture() -> ContainerPolicy {
        serde_json::from_str(include_str!("../fixtures/container-policy.json"))
            .expect("the policy fixture")
    }

    fn with(edit: impl FnOnce(&mut Value)) -> bool {
        let mut value: Value =
            serde_json::from_str(include_str!("../fixtures/container-policy.json")).expect("JSON");
        edit(&mut value);
        serde_json::from_value::<ContainerPolicy>(value).is_ok()
    }

    fn nonzero32(n: u32) -> NonZeroU32 {
        NonZeroU32::new(n).expect("nonzero")
    }

    fn nonzero64(n: u64) -> NonZeroU64 {
        NonZeroU64::new(n).expect("nonzero")
    }

    #[test]
    fn every_policy_field_changes_the_hash() {
        let base = fixture();
        // No `..`: a new field breaks this pattern, so its author has to add an edit below.
        let ContainerPolicy {
            image: _,
            user: _,
            cpus_milli: _,
            memory_bytes: _,
            pids: _,
            timeout_ms: _,
            tmp_bytes: _,
            network: _,
            root: _,
            privileges: _,
            mounts: _,
        } = &base;
        type Edit = fn(&mut ContainerPolicy);
        let edits: [(&str, Edit); 15] = [
            ("image name", |p| {
                p.image.name = "oven/bun:2".parse().expect("a name");
            }),
            ("image digest", |p| p.image.digest = None),
            ("uid", |p| p.user.uid = nonzero32(1001)),
            ("gid", |p| p.user.gid = nonzero32(1001)),
            ("cpus", |p| p.cpus_milli = nonzero32(1000)),
            ("memory", |p| p.memory_bytes = nonzero64(512 << 20)),
            ("pids", |p| p.pids = nonzero32(128)),
            ("timeout", |p| p.timeout_ms = nonzero64(60_000)),
            ("tmp", |p| p.tmp_bytes = nonzero64(64 << 20)),
            ("network", |p| p.network = ContainerNetwork::None {}),
            ("published port", |p| {
                p.network = ContainerNetwork::LoopbackPublished {
                    port: NonZeroU16::new(8080).expect("nonzero"),
                };
            }),
            ("workdir", |p| {
                p.mounts.workdir = "/app".parse().expect("a path");
            }),
            ("protected paths", |p| {
                p.mounts
                    .protected
                    .insert("src/limits.ts".parse().expect("a path"));
            }),
            ("missing protected", |p| {
                p.mounts.missing_protected = MissingProtected::Refused;
            }),
            ("readonly git", |p| p.mounts.readonly_git = false),
        ];
        let original = base.sandbox_policy_hash().expect("hashing");
        let mut seen = BTreeSet::from([original]);
        for (field, edit) in &edits {
            let mut changed = base.clone();
            edit(&mut changed);
            let hash = changed.sandbox_policy_hash().expect("hashing");
            assert!(seen.insert(hash), "changing the {field} left a known hash");
        }
    }

    #[test]
    fn a_run_is_never_root() {
        assert!(with(|_| {}));
        for user in [
            "0:1000",
            "1000:0",
            "root",
            "1000",
            "+1000:1000",
            "01000:1000",
            "1000:1000:1",
            "-1:1",
            "99999999999:1",
        ] {
            assert!(!with(|v| v["user"] = json!(user)), "{user} decoded");
        }
        let user: ContainerUser = "1000:1000".parse().expect("a user");
        assert_eq!(user.to_string(), "1000:1000");
    }

    #[test]
    fn image_names_cannot_read_as_flags_or_carry_a_digest() {
        for name in [
            "",
            "-v",
            "--privileged",
            "bun latest",
            "oven/bun@sha256:00",
            "a,b",
        ] {
            assert!(
                !with(|v| v["image"]["name"] = json!(name)),
                "{name:?} decoded"
            );
        }
        assert!(!with(|v| v["image"]["digest"] = json!("sha256:short")));
        assert!(with(|v| {
            v["image"]
                .as_object_mut()
                .expect("an image")
                .remove("digest");
        }));
    }

    #[test]
    fn caps_and_paths_are_bounded() {
        for cap in [
            "cpus_milli",
            "memory_bytes",
            "pids",
            "timeout_ms",
            "tmp_bytes",
        ] {
            assert!(!with(|v| v[cap] = json!(0)), "a zero {cap} decoded");
        }
        for workdir in [
            "work",
            "/",
            "/work/..",
            "/work/./x",
            "/work dir",
            "/work//x",
        ] {
            assert!(
                !with(|v| v["mounts"]["workdir"] = json!(workdir)),
                "{workdir} decoded"
            );
        }
        assert!(!with(|v| v["mounts"]["protected"] = json!(["../outside"])));
        assert!(!with(
            |v| v["mounts"]["missing_protected"] = json!("writable")
        ));
        assert!(!with(|v| v["mounts"]["readonly_git"] = json!("no")));
    }

    #[test]
    fn readonly_git_defaults_to_true_and_is_hashed() {
        let base = fixture();
        assert!(base.mounts.readonly_git);
        let mut absent: Value =
            serde_json::from_str(include_str!("../fixtures/container-policy.json")).expect("JSON");
        absent["mounts"]
            .as_object_mut()
            .expect("mounts")
            .remove("readonly_git");
        let decoded: ContainerPolicy = serde_json::from_value(absent).expect("the policy decodes");
        assert_eq!(decoded, base, "an absent readonly_git is true");
        let mut opened = base.clone();
        opened.mounts.readonly_git = false;
        assert_ne!(
            opened.sandbox_policy_hash().expect("hashing"),
            base.sandbox_policy_hash().expect("hashing"),
            "a writable .git kept the policy hash"
        );
    }

    #[test]
    fn the_network_is_closed() {
        assert!(with(|v| v["network"] = json!({ "kind": "none" })));
        assert!(!with(
            |v| v["network"] = json!({ "kind": "none", "port": 3000 })
        ));
        assert!(!with(|v| v["network"] = json!({ "kind": "host" })));
        assert!(!with(
            |v| v["network"] = json!({ "kind": "loopback_published", "port": 0 })
        ));
        assert!(!with(|v| v["root"] = json!("read_write")));
    }
}
