use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    TypeScript,
    Tsx,
    JavaScript,
    Rust,
}

impl Lang {
    pub(crate) const ALL: [Lang; 4] = [Lang::TypeScript, Lang::Tsx, Lang::JavaScript, Lang::Rust];

    /// Detects the language from a repo-relative path. Returns `None` for files the scanner does not read.
    pub fn from_path(path: &str) -> Option<Lang> {
        match Path::new(path).extension()?.to_str()? {
            "ts" | "mts" | "cts" => Some(Lang::TypeScript),
            "tsx" => Some(Lang::Tsx),
            "js" | "jsx" | "mjs" | "cjs" => Some(Lang::JavaScript),
            "rs" => Some(Lang::Rust),
            _ => None,
        }
    }

    pub(crate) fn language(self) -> tree_sitter::Language {
        match self {
            Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
        }
    }
}
