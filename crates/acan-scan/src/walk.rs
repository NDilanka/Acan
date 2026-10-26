use std::path::{Component, Path, PathBuf};

use ignore::WalkBuilder;

use crate::ScanError;
use crate::lang::Lang;

/// Directories that hold generated or vendored code, never repo source.
const SKIPPED_DIRS: [&str; 3] = ["node_modules", "target", "dist"];

pub(crate) struct SourceEntry {
    pub(crate) abs: PathBuf,
    /// Repo-relative, forward slashes.
    pub(crate) rel: String,
    pub(crate) lang: Lang,
}

/// Lists the scannable files under `root`, honoring `.gitignore`, sorted by repo-relative path.
pub(crate) fn source_files(root: &Path) -> Result<Vec<SourceEntry>, ScanError> {
    let walker = WalkBuilder::new(root)
        .require_git(false)
        .filter_entry(|entry| {
            let skipped_dir = entry.depth() > 0
                && entry.file_type().is_some_and(|kind| kind.is_dir())
                && SKIPPED_DIRS.iter().any(|dir| entry.file_name() == *dir);
            !skipped_dir
        })
        .build();

    let mut sources = Vec::new();
    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        let rel = relative_path(root, entry.path())?;
        if let Some(lang) = Lang::from_path(&rel) {
            sources.push(SourceEntry {
                abs: entry.into_path(),
                rel,
                lang,
            });
        }
    }
    sources.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(sources)
}

/// Joins the components with `/` so the same repo gives the same paths on Windows and Linux.
fn relative_path(root: &Path, path: &Path) -> Result<String, ScanError> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let mut parts = Vec::new();
    for component in relative.components() {
        if let Component::Normal(part) = component {
            let part = part
                .to_str()
                .ok_or_else(|| ScanError::NonUtf8Path(path.to_path_buf()))?;
            parts.push(part);
        }
    }
    Ok(parts.join("/"))
}
