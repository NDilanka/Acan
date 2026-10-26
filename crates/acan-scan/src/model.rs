use std::collections::BTreeMap;
use std::ops::Range;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::lang::Lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plant {
    pub root: PathBuf,
    pub machines: Vec<Machine>,
    pub belts: Vec<Belt>,
    pub files: Vec<SourceFile>,
}

/// A replaceable module: a set of files that the foreman can regroup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Machine {
    pub id: MachineId,
    pub name: String,
    /// Common ancestor directory of the member files, repo-relative with forward slashes. `.` for the root.
    pub dir: String,
    pub files: Vec<FileId>,
    pub lang_mix: BTreeMap<Lang, u32>,
}

/// Import edges from one machine to another, aggregated from file-level imports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Belt {
    pub from: MachineId,
    pub to: MachineId,
    pub imports: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub id: FileId,
    /// Repo-relative, forward slashes.
    pub path: String,
    pub lang: Lang,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<ImportRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub byte_range: Range<usize>,
    pub line_range: LineRange,
    pub exported: bool,
}

/// 1-based, inclusive on both ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Struct,
    Enum,
    Trait,
    Interface,
    Type,
    Const,
    /// A `let` or `var` binding that is not a function.
    Variable,
    /// An exported name that has no local declaration: a re-export, a `default` export, or an export of an import.
    Export,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRef {
    /// The module specifier as written, or the expanded `use` path for Rust.
    pub specifier: String,
    pub line: u32,
    pub target: ImportTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ImportTarget {
    /// Resolved to a scanned file in this repo.
    File { file: FileId },
    /// Package or crate outside this repo. Never a belt.
    External,
    /// A relative or crate-local path that does not resolve to a scanned file.
    Unresolved,
}
