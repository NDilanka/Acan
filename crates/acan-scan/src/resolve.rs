use std::collections::HashMap;

use crate::extract::{RawImport, RawRef};
use crate::model::{FileId, ImportRef, ImportTarget};

/// Repo-relative path to file id, for the files that were scanned.
pub(crate) struct PathIndex(HashMap<String, FileId>);

impl PathIndex {
    pub(crate) fn from_paths<'a>(paths: impl IntoIterator<Item = &'a str>) -> Self {
        Self(
            paths
                .into_iter()
                .enumerate()
                .map(|(index, path)| (path.to_string(), FileId(index as u32)))
                .collect(),
        )
    }

    fn get(&self, path: &str) -> Option<FileId> {
        self.0.get(path).copied()
    }
}

pub(crate) fn resolve_import(from: &str, raw: RawImport, index: &PathIndex) -> ImportRef {
    let (specifier, target) = match raw.reference {
        RawRef::Module(specifier) => {
            let target = resolve_module(from, &specifier, index);
            (specifier, target)
        }
        RawRef::RustMod(name) => {
            let target = resolve_rust_mod(from, &name, index);
            (name, target)
        }
        RawRef::RustUse(path) => {
            let target = resolve_rust_use(from, &path, index);
            (path, target)
        }
    };
    ImportRef {
        specifier,
        line: raw.line,
        target,
    }
}

const JS_EXTENSIONS: [&str; 9] = [
    ".ts", ".tsx", ".d.ts", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs",
];
const TS_FOR_JS_EXTENSIONS: [&str; 5] = [".ts", ".tsx", ".mts", ".cts", ".d.ts"];
const JS_SOURCE_SUFFIXES: [&str; 4] = [".js", ".jsx", ".mjs", ".cjs"];

/// Bare specifiers are packages, so they are external. Relative specifiers must resolve inside the repo.
fn resolve_module(from: &str, specifier: &str, index: &PathIndex) -> ImportTarget {
    if !is_relative(specifier) {
        return ImportTarget::External;
    }
    let Some(base) = normalize(&join(parent_dir(from), specifier)) else {
        return ImportTarget::Unresolved;
    };
    find_file(index, js_candidates(&base))
}

fn is_relative(specifier: &str) -> bool {
    matches!(specifier, "." | "..") || specifier.starts_with("./") || specifier.starts_with("../")
}

/// Order follows TypeScript. It tries the path as written, then extensions, then `index` files, then a `.js` specifier mapped to its `.ts` source.
fn js_candidates(base: &str) -> Vec<String> {
    let mut candidates = vec![base.to_string()];
    candidates.extend(JS_EXTENSIONS.iter().map(|ext| format!("{base}{ext}")));
    candidates.extend(
        JS_EXTENSIONS
            .iter()
            .map(|ext| join(base, &format!("index{ext}"))),
    );
    if let Some(stem) = JS_SOURCE_SUFFIXES
        .iter()
        .find_map(|suffix| base.strip_suffix(suffix))
    {
        candidates.extend(
            TS_FOR_JS_EXTENSIONS
                .iter()
                .map(|ext| format!("{stem}{ext}")),
        );
    }
    candidates
}

/// `mod name;` lives in the module's own directory. `a/b.rs` declares into `a/b/`, while `mod.rs`, `lib.rs`, and `main.rs` declare into their own directory.
fn resolve_rust_mod(from: &str, name: &str, index: &PathIndex) -> ImportTarget {
    let base = join(&rust_module_dir(from), name);
    find_file(index, [format!("{base}.rs"), format!("{base}/mod.rs")])
}

fn rust_module_dir(from: &str) -> String {
    let dir = parent_dir(from);
    let file_name = from.rsplit('/').next().unwrap_or(from);
    match file_name {
        "mod.rs" | "lib.rs" | "main.rs" => dir.to_string(),
        _ => join(dir, file_name.strip_suffix(".rs").unwrap_or(file_name)),
    }
}

/// Resolves `crate::a::b::Item` by the longest module prefix that is a scanned file. `self::` and `super::` are left unresolved, and other roots are external crates.
fn resolve_rust_use(from: &str, path: &str, index: &PathIndex) -> ImportTarget {
    let mut segments = path.split("::");
    match segments.next() {
        Some("crate") => {}
        Some("self" | "super") => return ImportTarget::Unresolved,
        _ => return ImportTarget::External,
    }
    let Some(src_dir) = crate_src_dir(from) else {
        return ImportTarget::Unresolved;
    };
    let modules: Vec<&str> = segments
        .filter(|segment| !segment.is_empty() && *segment != "*")
        .collect();
    for depth in (0..=modules.len()).rev() {
        let candidates = if depth == 0 {
            vec![join(src_dir, "lib.rs"), join(src_dir, "main.rs")]
        } else {
            let base = modules[..depth]
                .iter()
                .fold(src_dir.to_string(), |acc, module| join(&acc, module));
            vec![format!("{base}.rs"), format!("{base}/mod.rs")]
        };
        let target = find_file(index, candidates);
        if !matches!(target, ImportTarget::Unresolved) {
            return target;
        }
    }
    ImportTarget::Unresolved
}

/// The nearest ancestor directory named `src`, which is the crate root for `crate::` paths.
fn crate_src_dir(from: &str) -> Option<&str> {
    let mut dir = parent_dir(from);
    loop {
        if dir.rsplit('/').next() == Some("src") {
            return Some(dir);
        }
        if dir.is_empty() {
            return None;
        }
        dir = parent_dir(dir);
    }
}

fn find_file(index: &PathIndex, candidates: impl IntoIterator<Item = String>) -> ImportTarget {
    candidates
        .into_iter()
        .find_map(|candidate| index.get(&candidate))
        .map_or(ImportTarget::Unresolved, |file| ImportTarget::File { file })
}

fn parent_dir(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(dir, _)| dir)
}

fn join(dir: &str, rest: &str) -> String {
    if dir.is_empty() {
        rest.to_string()
    } else {
        format!("{dir}/{rest}")
    }
}

/// Removes `.` and resolves `..`. Returns `None` when the path climbs above the repo root.
fn normalize(path: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}
