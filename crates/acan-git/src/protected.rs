//! The paths no candidate can change: `.acan` and whatever else the caller protects.

use std::collections::BTreeSet;

use acan_intent::RepoPath;

/// The directory of protected inputs every repository has: probes, the app contract,
/// scopes and rulings.
const ACAN_DIR: &str = ".acan";

/// The protected paths of a repository. Each covers itself and everything under it.
///
/// `.acan` is always included. Matching ignores case, because a case-insensitive
/// checkout writes `.ACAN/x` into `.acan/x`; a candidate's case variant of a protected
/// path is therefore treated as that path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtectedPaths {
    paths: BTreeSet<RepoPath>,
}

impl ProtectedPaths {
    /// `.acan` and `paths`, such as the container policy's protected mounts.
    #[must_use]
    pub fn new(paths: impl IntoIterator<Item = RepoPath>) -> Self {
        let mut paths: BTreeSet<RepoPath> = paths.into_iter().collect();
        paths.insert(ACAN_DIR.parse().expect(".acan is a portable repo path"));
        Self { paths }
    }

    /// The protected paths, sorted.
    pub fn iter(&self) -> impl Iterator<Item = &RepoPath> {
        self.paths.iter()
    }

    /// Whether `path`, a repo-relative path with `/` separators, is a protected path or
    /// lies under one, ignoring case.
    #[must_use]
    pub fn covers(&self, path: &str) -> bool {
        let path = path.to_lowercase();
        self.folded()
            .any(|protected| path == protected || is_under(&path, &protected))
    }

    /// Whether `path` is a proper ancestor of a protected path, ignoring case. Such a path
    /// must be a real directory, or a file or link there would redirect the protected path.
    pub(crate) fn encloses(&self, path: &str) -> bool {
        let path = path.to_lowercase();
        self.folded().any(|protected| is_under(&protected, &path))
    }

    fn folded(&self) -> impl Iterator<Item = String> + '_ {
        self.paths.iter().map(|path| path.as_str().to_lowercase())
    }
}

impl Default for ProtectedPaths {
    /// Only `.acan`.
    fn default() -> Self {
        Self::new([])
    }
}

/// Whether `path` lies strictly under `dir`, segment-wise.
fn is_under(path: &str, dir: &str) -> bool {
    path.strip_prefix(dir)
        .is_some_and(|rest| rest.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(raw: &[&str]) -> ProtectedPaths {
        ProtectedPaths::new(raw.iter().map(|p| p.parse().expect("a repo path")))
    }

    #[test]
    fn acan_is_always_protected_and_matching_is_segment_wise() {
        let protected = paths(&["test/login.test.ts"]);
        assert!(protected.covers(".acan"));
        assert!(protected.covers(".acan/probes/login.json"));
        assert!(protected.covers(".ACAN/probes/login.json"));
        assert!(protected.covers("Test/Login.test.ts"));
        assert!(!protected.covers(".acanfoo/x"));
        assert!(!protected.covers("test/login.test.ts.bak"));
        assert!(!protected.covers("test/other.test.ts"));
    }

    #[test]
    fn only_proper_ancestors_enclose_a_protected_path() {
        let protected = paths(&["test/unit/login.test.ts"]);
        assert!(protected.encloses("test"));
        assert!(protected.encloses("TEST/unit"));
        assert!(!protected.encloses("test/unit/login.test.ts"));
        assert!(!protected.encloses("tes"));
        assert!(!protected.encloses("src"));
    }
}
