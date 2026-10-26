//! Scans a source repository with tree-sitter into a plant: machines, the belts between them,
//! and per-file symbols and imports for clash detection.

mod extract;
mod group;
mod lang;
mod model;
mod resolve;
mod walk;

use std::path::{Path, PathBuf};

pub use group::Grouping;
pub use lang::Lang;
pub use model::{
    Belt, FileId, ImportRef, ImportTarget, LineRange, Machine, MachineId, Plant, SourceFile,
    Symbol, SymbolKind,
};

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error(transparent)]
    Walk(#[from] ignore::Error),
    #[error("path is not valid UTF-8: {}", .0.display())]
    NonUtf8Path(PathBuf),
    #[error("failed to read {}: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to load the {lang:?} grammar: {source}")]
    Language {
        lang: Lang,
        #[source]
        source: tree_sitter::LanguageError,
    },
    #[error("failed to compile the {lang:?} query: {source}")]
    Query {
        lang: Lang,
        #[source]
        source: tree_sitter::QueryError,
    },
    #[error("parsing {path} did not finish")]
    ParseCancelled { path: String },
}

/// Scans the repo at `root` and returns its plant, grouped by directory.
pub fn scan(root: impl AsRef<Path>) -> Result<Plant, ScanError> {
    let root = root.as_ref();
    let sources = walk::source_files(root)?;
    let index = resolve::PathIndex::from_paths(sources.iter().map(|source| source.rel.as_str()));
    let mut extractor = extract::Extractor::new()?;

    let mut files = Vec::with_capacity(sources.len());
    for (position, source) in sources.iter().enumerate() {
        let text = std::fs::read_to_string(&source.abs).map_err(|error| ScanError::Read {
            path: source.abs.clone(),
            source: error,
        })?;
        let extracted = extractor.extract(source.lang, &source.rel, &text)?;
        let imports = extracted
            .imports
            .into_iter()
            .map(|raw| resolve::resolve_import(&source.rel, raw, &index))
            .collect();
        files.push(SourceFile {
            id: FileId(position as u32),
            path: source.rel.clone(),
            lang: source.lang,
            symbols: extracted.symbols,
            imports,
        });
    }

    let mut plant = Plant {
        root: root.to_path_buf(),
        machines: Vec::new(),
        belts: Vec::new(),
        files,
    };
    plant.regroup(&Grouping::default());
    Ok(plant)
}
