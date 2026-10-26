//! The code map the clash detector reads: files, their symbols, and the belts between machines.
//!
//! The host builds a [`CodeIndex`] from its scanner and passes it in [`crate::Ctx`]. Only the
//! origin reads it, inside `execute`; replicas never need it because detection results are
//! recorded as events.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::ids::{CodeRev, FilePath, MachineKey};

/// A snapshot of the repository's files, symbols and machine import edges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeIndex {
    /// Content revision of this snapshot. Equal revisions must mean equal contents.
    pub rev: CodeRev,
    /// Every scanned file, keyed by its repo-relative path.
    pub files: BTreeMap<FilePath, IndexedFile>,
    /// Import edges between machines. A belt `from -> to` means `from` imports `to`.
    pub belts: BTreeSet<Belt>,
}

/// One scanned file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedFile {
    /// The machine this file belongs to.
    pub machine: MachineKey,
    /// Top-level and nested symbols, in any order. Names may repeat within a file.
    pub symbols: Vec<IndexedSymbol>,
}

/// A named symbol and the bytes it spans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedSymbol {
    /// The symbol name as written in source.
    pub name: String,
    /// Half-open byte range in the file at [`CodeIndex::rev`].
    pub bytes: Range<u32>,
    /// Whether other files can import this symbol. Contract overlaps only consider exported symbols.
    pub exported: bool,
}

/// An import edge between two machines.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Belt {
    /// The importing machine.
    pub from: MachineKey,
    /// The imported machine.
    pub to: MachineKey,
}

/// A place on the plant: what a readback touches, where a question or clash pin sits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "at", rename_all = "snake_case")]
pub enum Locus {
    /// A whole machine.
    Machine {
        /// The machine.
        machine: MachineKey,
    },
    /// The import edge from one machine to another.
    Belt {
        /// The importing machine.
        from: MachineKey,
        /// The imported machine.
        to: MachineKey,
    },
    /// A whole file.
    File {
        /// The file.
        file: FilePath,
    },
    /// A named symbol in a file. Keyed by name, not range, so it survives a rescan.
    Symbol {
        /// The file.
        file: FilePath,
        /// The symbol name.
        name: String,
    },
}

impl CodeIndex {
    /// Creates an empty index at `rev`. Every symbol reference against it is unresolved.
    #[must_use]
    pub fn new(rev: CodeRev) -> Self {
        Self {
            rev,
            files: BTreeMap::new(),
            belts: BTreeSet::new(),
        }
    }

    /// Returns every symbol named `name` in `file`. Empty when the file or name is unknown.
    pub fn symbols<'a>(
        &'a self,
        file: &FilePath,
        name: &'a str,
    ) -> impl Iterator<Item = &'a IndexedSymbol> + 'a {
        self.files
            .get(file)
            .into_iter()
            .flat_map(|f| f.symbols.iter())
            .filter(move |s| s.name == name)
    }

    /// Returns the machine that owns `file`, if the file was scanned.
    #[must_use]
    pub fn machine_of(&self, file: &FilePath) -> Option<&MachineKey> {
        self.files.get(file).map(|f| &f.machine)
    }
}
