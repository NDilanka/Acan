//! Maps a scanned machine to the model kind that draws it. Pure Rust, so the rules are unit
//! tested on the host and the same table serves live, fixture and stress plants.

/// A model kind. The stand-in draws one model per kind; acan-iso will replace the models.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Intake,
    Gate,
    Tank,
    Splitter,
    Valve,
    Assembler,
    Crate,
}

impl Kind {
    /// Every kind, in model index order.
    pub const ALL: [Kind; 7] = [
        Kind::Intake,
        Kind::Gate,
        Kind::Tank,
        Kind::Splitter,
        Kind::Valve,
        Kind::Assembler,
        Kind::Crate,
    ];

    /// Index into `ALL`, which is also the index of the kind's model.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Plan footprint `(width, depth)`. Crates are small; every other kind uses the
    /// assembler footprint until acan-iso supplies real ones.
    pub fn footprint(self) -> (f32, f32) {
        match self {
            Kind::Crate => (40.0, 40.0),
            _ => (96.0, 86.0),
        }
    }
}

/// Name keywords for each kind, checked in this order. The first kind with a keyword in the
/// machine name wins; the directory is checked only when the name matches nothing.
/// `ratelimit` is listed for readability even though `limit` already matches it.
const RULES: [(Kind, &[&str]); 5] = [
    (Kind::Intake, &["server", "main", "index"]),
    (Kind::Gate, &["auth", "session", "login"]),
    (Kind::Tank, &["store", "db", "repo"]),
    (Kind::Splitter, &["middleware", "proxy", "gateway"]),
    (Kind::Valve, &["ratelimit", "limit"]),
];

/// The kind of a machine, from its name, its directory and the paths of its files.
///
/// A machine with no files, or whose files are all tests, is a crate. Otherwise a keyword in
/// the name or the directory picks the kind, and anything left is an assembler.
pub fn classify(name: &str, dir: &str, file_paths: &[&str]) -> Kind {
    if file_paths.is_empty() || file_paths.iter().all(|path| is_test(path)) {
        return Kind::Crate;
    }
    let name = name.to_ascii_lowercase();
    let dir = dir.to_ascii_lowercase();
    for words in [&name, &dir] {
        for (kind, keywords) in RULES {
            if keywords.iter().any(|word| words.contains(word)) {
                return kind;
            }
        }
    }
    Kind::Assembler
}

fn is_test(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    let in_test_dir = lower
        .split(['/', '\\'])
        .any(|part| matches!(part, "test" | "tests" | "__tests__" | "spec"));
    let test_file =
        lower.contains(".test.") || lower.contains(".spec.") || lower.contains("_test.");
    in_test_dir || test_file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_pick_their_kind() {
        let src = ["src/x.ts"];
        assert_eq!(classify("server", "src", &src), Kind::Intake);
        assert_eq!(classify("main", "src", &src), Kind::Intake);
        assert_eq!(classify("index", "src", &src), Kind::Intake);
        assert_eq!(classify("auth", "src", &src), Kind::Gate);
        assert_eq!(classify("session", "src", &src), Kind::Gate);
        assert_eq!(classify("login", "src", &src), Kind::Gate);
        assert_eq!(classify("store", "src", &src), Kind::Tank);
        assert_eq!(classify("db", "src", &src), Kind::Tank);
        assert_eq!(classify("repo", "src", &src), Kind::Tank);
        assert_eq!(classify("middleware", "src", &src), Kind::Splitter);
        assert_eq!(classify("proxy", "src", &src), Kind::Splitter);
        assert_eq!(classify("gateway", "src", &src), Kind::Splitter);
        assert_eq!(classify("ratelimit", "src", &src), Kind::Valve);
        assert_eq!(classify("limit", "src", &src), Kind::Valve);
    }

    #[test]
    fn other_code_is_an_assembler() {
        let src = ["src/errors.ts"];
        assert_eq!(classify("errors", "src", &src), Kind::Assembler);
        assert_eq!(classify("src", "src", &["src/auth.ts"]), Kind::Assembler);
    }

    #[test]
    fn the_name_decides_before_the_directory() {
        assert_eq!(classify("errors", "auth", &["auth/x.ts"]), Kind::Gate);
        assert_eq!(classify("errors", "store", &["store/x.ts"]), Kind::Tank);
    }

    #[test]
    fn test_only_and_unrecognized_machines_are_crates() {
        assert_eq!(
            classify("test", "test", &["test/a.ts", "test/b.ts"]),
            Kind::Crate
        );
        assert_eq!(classify("auth", "src", &["src/auth.test.ts"]), Kind::Crate);
        assert_eq!(classify("store", "src", &[]), Kind::Crate);
    }

    #[test]
    fn a_mixed_machine_is_not_test_only() {
        assert_eq!(
            classify("auth", "src", &["src/auth.ts", "src/auth.test.ts"]),
            Kind::Gate
        );
    }

    #[test]
    fn footprints_match_the_brief() {
        assert_eq!(Kind::Assembler.footprint(), (96.0, 86.0));
        assert_eq!(Kind::Crate.footprint(), (40.0, 40.0));
        for kind in Kind::ALL {
            assert_eq!(Kind::ALL[kind.index()], kind);
        }
    }
}
