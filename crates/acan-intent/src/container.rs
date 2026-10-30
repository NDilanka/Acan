//! The container policy of every isolated run: the candidate app, every probe sequence, and the
//! worker's test command.
//!
//! Policy rules, binding on every runner:
//! - The command runs as a non-root user on a read-only root filesystem with a bounded `/tmp`,
//!   with no capabilities and no privilege gain, under the CPU, memory and process caps below.
//!   A run is killed and removed when it ends or when its time limit passes: `timeout_ms` for one
//!   probe sequence or test command, `role_timeout_ms` for a long-lived role container.
//! - The worktree is mounted read-write at the working directory. Every protected path is
//!   mounted read-only on top of it from a snapshot of the protected head, never from the
//!   worktree. A protected path the snapshot does not have is still covered as
//!   [`MissingProtected`] says; it is never left writable.
//! - The container has no network and publishes no port. Probes run inside it from the harness
//!   mounted at `mounts.harness`; the host drives them over the container's standard input and
//!   output.
//! - The container gets only the variables `environment` names, with the values it gives them.
//!   No provider key, owner token or host secret enters it, and a name that could carry one is
//!   refused when the policy is decoded.
//! - A runner that cannot apply every field refuses the run. It never runs with less isolation.
//!   The harness is a probe-run field: the worker's test command runs no probe and mounts none.
//!
//! The policy's [`SandboxPolicyHash`] is part of the [`crate::GateContext`], so changing any
//! field voids a clearance obtained under the old policy.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};
use std::str::FromStr;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::{CanonicalError, ContentHash, ImageDigest, SandboxPolicyHash};
use crate::types::{ContainerPath, ImageName, InvalidValue, RepoPath};

/// The digest of `oven/bun:1.4.4`, the image the H01 spike ran. Every policy that names this image
/// pins this digest, and the runners refuse an image whose content differs.
const PINNED_IMAGE_DIGEST: &str =
    "sha256:1d22548f177f4ff38e0c15baa3c2c46c6c2060ba2d1bc573bb4720b407548f1d";

/// Fragments that mark a variable as a credential. No variable whose name contains one reaches a
/// container.
const CREDENTIAL_NAME_PARTS: [&str; 7] = [
    "KEY",
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "CREDENTIAL",
    "AUTH",
];
/// Longest variable name accepted.
const ENVIRONMENT_NAME_MAX: usize = 64;
/// Longest variable value accepted, in bytes.
const ENVIRONMENT_VALUE_MAX: usize = 4096;

/// The image a run starts from.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerImage {
    /// The name and tag, such as `oven/bun:1.4.4`.
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

/// What a run's container may reach. Closed: the container has no network at all, so nothing is
/// published and probes run inside it.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContainerNetwork {
    /// No network at all. Probes and the test command run this way.
    None {},
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
    /// Where the probe driver's directory is mounted read-only. The driver runs inside the
    /// container and the host drives it over standard input. Only probe runs mount it.
    #[serde(default = "harness_by_default")]
    pub harness: ContainerPath,
}

/// The default for [`MountPolicy::readonly_git`]: `.git` is mounted read-only.
const fn readonly_git_by_default() -> bool {
    true
}

/// The default for [`MountPolicy::harness`]: the probe driver is mounted at `/harness`.
fn harness_by_default() -> ContainerPath {
    "/harness"
        .parse()
        .expect("the default harness path is a container path")
}

/// The default for [`ContainerPolicy::role_timeout_ms`]: one hour.
fn role_timeout_by_default() -> NonZeroU64 {
    nonzero64(3_600_000)
}

/// The default for [`ContainerPolicy::environment`]: only `HOME`, which is the writable `/tmp`.
fn environment_by_default() -> BTreeMap<String, String> {
    BTreeMap::from([("HOME".to_owned(), "/tmp".to_owned())])
}

/// Decodes the environment and refuses any variable a container must not receive.
fn environment_names<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, String>, D::Error> {
    let environment = BTreeMap::<String, String>::deserialize(deserializer)?;
    for (name, value) in &environment {
        check_variable(name, value).map_err(serde::de::Error::custom)?;
    }
    Ok(environment)
}

/// Refuses a variable whose name is not `A-Z 0-9 _` from an uppercase letter, whose name could
/// carry a credential, or whose value is not short printable text.
fn check_variable(name: &str, value: &str) -> Result<(), String> {
    let well_formed = name.len() <= ENVIRONMENT_NAME_MAX
        && name.starts_with(|c: char| c.is_ascii_uppercase())
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !well_formed {
        return Err(format!("{name:?} is not an environment variable name"));
    }
    if CREDENTIAL_NAME_PARTS.iter().any(|part| name.contains(part)) {
        return Err(format!(
            "{name} may carry a credential, so no container may receive it"
        ));
    }
    if value.len() > ENVIRONMENT_VALUE_MAX || value.chars().any(char::is_control) {
        return Err(format!(
            "the value of {name} is not printable text of at most {ENVIRONMENT_VALUE_MAX} bytes"
        ));
    }
    Ok(())
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
    /// Wall-clock cap of one probe sequence or one test command, from container start. At it
    /// the runner kills and removes the container.
    pub timeout_ms: NonZeroU64,
    /// Wall-clock cap of a long-lived role container. It is separate from `timeout_ms`, so a
    /// role container is not cut at the sequence cap.
    #[serde(default = "role_timeout_by_default")]
    pub role_timeout_ms: NonZeroU64,
    /// Size of the writable `/tmp`, which is also `HOME`.
    pub tmp_bytes: NonZeroU64,
    /// The environment variables the container gets, by name, with their values. A variable not
    /// listed never reaches the container.
    #[serde(
        default = "environment_by_default",
        deserialize_with = "environment_names"
    )]
    pub environment: BTreeMap<String, String>,
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
    /// The policy a run starts under when none is named: the contract fixture
    /// `container-policy.json`, which the worker's test command also runs under. The contract
    /// test `the_fixture_is_the_default_run` holds the two equal.
    #[must_use]
    pub fn default_run() -> Self {
        Self {
            image: ContainerImage {
                name: "oven/bun:1.4.4"
                    .parse()
                    .expect("the pinned image name is valid"),
                digest: Some(
                    PINNED_IMAGE_DIGEST
                        .parse()
                        .expect("the pinned image digest is valid"),
                ),
            },
            user: "1000:1000".parse().expect("the run user is valid"),
            cpus_milli: nonzero32(2000),
            memory_bytes: nonzero64(1 << 30),
            pids: nonzero32(256),
            timeout_ms: nonzero64(30_000),
            role_timeout_ms: role_timeout_by_default(),
            tmp_bytes: nonzero64(256 << 20),
            environment: BTreeMap::from([
                ("HOME".to_owned(), "/tmp".to_owned()),
                ("PORT".to_owned(), "3000".to_owned()),
            ]),
            network: ContainerNetwork::None {},
            root: RootFilesystem::ReadOnly,
            privileges: Privileges::Dropped,
            mounts: MountPolicy {
                workdir: "/work".parse().expect("the workdir is a container path"),
                protected: BTreeSet::from([".acan".parse().expect("a repo path")]),
                missing_protected: MissingProtected::Masked,
                readonly_git: true,
                harness: harness_by_default(),
            },
        }
    }

    /// Refuses an environment a container must not get. The decoder applies the same rule, so a
    /// policy built in code is checked before any runner passes its variables on.
    ///
    /// # Errors
    ///
    /// Returns the reason for the first variable that fails.
    pub fn check_environment(&self) -> Result<(), String> {
        self.environment
            .iter()
            .try_for_each(|(name, value)| check_variable(name, value))
    }

    /// The policy's identity, carried in [`crate::GateContext::sandbox_policy_hash`].
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn sandbox_policy_hash(&self) -> Result<SandboxPolicyHash, CanonicalError> {
        ContentHash::of(self).map(SandboxPolicyHash::from)
    }
}

fn nonzero32(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).expect("a container limit is never zero")
}

fn nonzero64(n: u64) -> NonZeroU64 {
    NonZeroU64::new(n).expect("a container limit is never zero")
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
            role_timeout_ms: _,
            tmp_bytes: _,
            environment: _,
            network: _,
            root: _,
            privileges: _,
            mounts: _,
        } = &base;
        type Edit = fn(&mut ContainerPolicy);
        let edits: [(&str, Edit); 17] = [
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
            ("role timeout", |p| p.role_timeout_ms = nonzero64(7_200_000)),
            ("tmp", |p| p.tmp_bytes = nonzero64(64 << 20)),
            ("an added variable", |p| {
                p.environment.insert("TZ".to_owned(), "UTC".to_owned());
            }),
            ("a variable's value", |p| {
                p.environment
                    .insert("HOME".to_owned(), "/var/tmp".to_owned());
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
            ("harness", |p| {
                p.mounts.harness = "/sandbox".parse().expect("a path");
            }),
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
            "role_timeout_ms",
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
        for harness in ["harness", "/harness/..", "/harness dir"] {
            assert!(
                !with(|v| v["mounts"]["harness"] = json!(harness)),
                "{harness} decoded"
            );
        }
        assert!(!with(|v| v["mounts"]["protected"] = json!(["../outside"])));
        assert!(!with(
            |v| v["mounts"]["missing_protected"] = json!("writable")
        ));
        assert!(!with(|v| v["mounts"]["readonly_git"] = json!("no")));
    }

    #[test]
    fn environment_names_refuse_credentials_and_control_text() {
        assert!(with(
            |v| v["environment"] = json!({ "HOME": "/tmp", "PORT": "3000" })
        ));
        assert!(with(|v| v["environment"] = json!({})));
        for name in [
            "OPENAI_API_KEY",
            "META_TOKEN",
            "DB_PASSWORD",
            "GITHUB_SECRET",
            "AUTH_HEADER",
            "my_var",
            "1PORT",
            "PORT=3000",
            "A B",
            "",
        ] {
            assert!(
                !with(|v| v["environment"] = json!({ name: "x" })),
                "{name:?} decoded"
            );
        }
        for value in ["3000\n", "a\u{7}b"] {
            assert!(
                !with(|v| v["environment"] = json!({ "PORT": value })),
                "{value:?} decoded"
            );
        }
    }

    #[test]
    fn a_missing_environment_or_harness_takes_the_probe_defaults() {
        let mut absent: Value =
            serde_json::from_str(include_str!("../fixtures/container-policy.json")).expect("JSON");
        absent
            .as_object_mut()
            .expect("a policy")
            .remove("environment");
        absent["mounts"]
            .as_object_mut()
            .expect("mounts")
            .remove("harness");
        absent
            .as_object_mut()
            .expect("a policy")
            .remove("role_timeout_ms");
        let decoded: ContainerPolicy = serde_json::from_value(absent).expect("the policy decodes");
        assert_eq!(
            decoded.environment,
            BTreeMap::from([("HOME".to_owned(), "/tmp".to_owned())]),
            "only HOME when the policy names nothing"
        );
        assert_eq!(decoded.mounts.harness.as_str(), "/harness");
        assert_eq!(decoded.role_timeout_ms, nonzero64(3_600_000));
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
            |v| v["network"] = json!({ "kind": "loopback_published", "port": 3000 })
        ));
        assert!(!with(|v| v["root"] = json!("read_write")));
    }

    #[test]
    fn the_default_run_pins_the_digest_the_spike_ran() {
        let image = ContainerPolicy::default_run().image;
        assert_eq!(image.name.as_str(), "oven/bun:1.4.4");
        assert_eq!(
            image.digest.map(|digest| digest.to_string()).as_deref(),
            Some(PINNED_IMAGE_DIGEST)
        );
    }
}
