//! Writing a prepared tree and its protected overlay to disk without following links.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use acan_intent::{ArtifactHash, ContentHash, OverlayManifest, RepoPath};

use crate::prepare::BlobFile;
use crate::{Error, ProtectedPaths};

/// Refuse a tree whose paths collide when case is ignored, including a file that shares a
/// folded name with a directory.
pub(crate) fn refuse_case_collisions<'a>(
    paths: impl IntoIterator<Item = &'a str>,
) -> Result<(), Error> {
    let mut files: HashSet<String> = HashSet::new();
    let mut dirs: HashMap<String, &'a str> = HashMap::new();
    for path in paths {
        let collision = || Error::CaseCollision(path.to_owned());
        for (end, _) in path.match_indices('/') {
            let dir = &path[..end];
            let folded = dir.to_lowercase();
            if files.contains(&folded) {
                return Err(collision());
            }
            if *dirs.entry(folded).or_insert(dir) != dir {
                return Err(collision());
            }
        }
        let folded = path.to_lowercase();
        if dirs.contains_key(&folded) || !files.insert(folded) {
            return Err(collision());
        }
    }
    Ok(())
}

/// Write `files` into `root`, which must be missing or empty.
pub(crate) fn write_files(root: &Path, files: &[(BlobFile, Vec<u8>)]) -> Result<(), Error> {
    fs::create_dir_all(root)?;
    if fs::read_dir(root)?.next().is_some() {
        return Err(Error::TargetNotEmpty(root.to_path_buf()));
    }
    for (file, bytes) in files {
        write_file(root, &file.path, bytes, file.executable)?;
    }
    Ok(())
}

/// Remove everything at every protected path of `root`, case variants included, write
/// `files` in their place, and return the manifest of what the protected paths now hold.
pub(crate) fn restore(
    root: &Path,
    protected: &ProtectedPaths,
    files: &[(BlobFile, Vec<u8>)],
) -> Result<OverlayManifest, Error> {
    fs::create_dir_all(root)?;
    for path in protected.iter() {
        let segments: Vec<&str> = path.as_str().split('/').collect();
        remove_matching(root, &segments)?;
    }
    for (file, bytes) in files {
        write_file(root, &file.path, bytes, file.executable)?;
    }
    let mut manifest = BTreeMap::new();
    for path in protected.iter() {
        collect(root, path.as_str(), &mut manifest)?;
    }
    Ok(OverlayManifest { files: manifest })
}

/// Remove every entry under `dir` whose path matches `segments` when case is ignored. An
/// entry on the way that is not a real directory is removed too, since it could redirect
/// the protected path.
fn remove_matching(dir: &Path, segments: &[&str]) -> io::Result<()> {
    let Some((first, rest)) = segments.split_first() else {
        return Ok(());
    };
    let first = first.to_lowercase();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        // A name that is not UTF-8 cannot fold to a protected segment, which is UTF-8.
        if name
            .to_str()
            .is_none_or(|name| name.to_lowercase() != first)
        {
            continue;
        }
        let path = entry.path();
        if !rest.is_empty() && fs::symlink_metadata(&path)?.is_dir() {
            remove_matching(&path, rest)?;
        } else {
            remove(&path)?;
        }
    }
    Ok(())
}

/// Add every file at or under `relative` to `manifest`. A link or any other non-file entry
/// is a mismatch, never followed.
fn collect(
    root: &Path,
    relative: &str,
    manifest: &mut BTreeMap<RepoPath, ArtifactHash>,
) -> Result<(), Error> {
    let full = join(root, relative);
    let meta = match fs::symlink_metadata(&full) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    if meta.is_file() {
        let hash = ContentHash::of_bytes(&fs::read(&full)?);
        manifest.insert(relative.parse()?, hash.into());
    } else if meta.is_dir() {
        for entry in fs::read_dir(&full)? {
            let name = entry?.file_name();
            let Some(name) = name.to_str() else {
                return Err(Error::NonUtf8Path(full.join(&name)));
            };
            collect(root, &format!("{relative}/{name}"), manifest)?;
        }
    } else {
        return Err(Error::OverlayMismatch(full));
    }
    Ok(())
}

/// Write one file, replacing whatever is at its path and making every parent a real
/// directory, so no link left in the tree can carry the write elsewhere.
fn write_file(root: &Path, path: &RepoPath, bytes: &[u8], executable: bool) -> io::Result<()> {
    let segments: Vec<&str> = path.as_str().split('/').collect();
    let (name, parents) = segments
        .split_last()
        .expect("a repo path has at least one segment");
    let mut dir = root.to_path_buf();
    for segment in parents {
        dir.push(segment);
        real_dir(&dir)?;
    }
    let target = dir.join(name);
    if fs::symlink_metadata(&target).is_ok() {
        remove(&target)?;
    }
    fs::write(&target, bytes)?;
    set_executable(&target, executable)
}

fn real_dir(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => Ok(()),
        Ok(_) => {
            remove(path)?;
            fs::create_dir(path)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => fs::create_dir(path),
        Err(e) => Err(e),
    }
}

/// Remove a file, a directory tree, or a link without following it.
fn remove(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if meta.is_dir() {
        fs::remove_dir_all(path)
    } else if meta.file_type().is_symlink() {
        // Windows removes a directory link or junction only with `remove_dir`.
        fs::remove_file(path).or_else(|_| fs::remove_dir(path))
    } else {
        fs::remove_file(path)
    }
}

fn join(root: &Path, relative: &str) -> PathBuf {
    let mut full = root.to_path_buf();
    full.extend(relative.split('/'));
    full
}

#[cfg(unix)]
fn set_executable(path: &Path, executable: bool) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if executable { 0o755 } else { 0o644 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path, _executable: bool) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_that_fold_together_are_refused() {
        assert!(refuse_case_collisions(["a/b.txt", "a/c.txt", "d.txt"]).is_ok());
        assert!(refuse_case_collisions(["README", "readme"]).is_err());
        assert!(refuse_case_collisions(["Src/a.ts", "src/b.ts"]).is_err());
        assert!(refuse_case_collisions(["lib", "LIB/x"]).is_err());
        assert!(refuse_case_collisions(["lib/x", "LIB"]).is_err());
    }
}
