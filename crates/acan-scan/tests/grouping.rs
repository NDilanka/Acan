use std::fs;
use std::path::Path;

use acan_scan::{Plant, scan};

fn write(root: &Path, path: &str, text: &str) {
    let full = root.join(path);
    fs::create_dir_all(full.parent().expect("a file has a parent")).expect("create directory");
    fs::write(full, text).expect("write source");
}

fn machine_names(plant: &Plant) -> Vec<&str> {
    plant.machines.iter().map(|m| m.name.as_str()).collect()
}

/// "from -> to xN" for each belt, using machine names.
fn belts(plant: &Plant) -> Vec<String> {
    plant
        .belts
        .iter()
        .map(|belt| {
            format!(
                "{} -> {} x{}",
                plant.machines[belt.from.0 as usize].name,
                plant.machines[belt.to.0 as usize].name,
                belt.imports
            )
        })
        .collect()
}

const SMALL_APP: &[(&str, &str)] = &[
    ("src/auth.ts", "export const session = 1;\n"),
    ("src/errors.ts", "export class HttpError {}\n"),
    (
        "src/links.ts",
        "import { HttpError } from './errors.ts';\nexport function links() { return HttpError; }\n",
    ),
    (
        "src/middleware.ts",
        "import { session } from './auth.ts';\nimport { HttpError } from './errors.ts';\nexport const gate = session;\n",
    ),
    ("src/ratelimit.ts", "export const limit = 1;\n"),
    (
        "src/server.ts",
        "import { gate } from './middleware.ts';\nimport { links } from './links.ts';\nexport const server = gate;\n",
    ),
    ("src/store.ts", "export const store = 1;\n"),
    (
        "src/dashboard.ts",
        "import { store } from './store.ts';\nexport const dash = store;\n",
    ),
    (
        "test/links.test.ts",
        "import { links } from '../src/links.ts';\nexport const t = links;\n",
    ),
    (
        "test/helpers.ts",
        "import { server } from '../src/server.ts';\nexport const h = server;\n",
    ),
];

#[test]
fn a_small_repo_gives_each_module_its_own_machine_and_keeps_tests_whole() {
    let dir = tempfile::tempdir().expect("temp dir");
    for (path, text) in SMALL_APP {
        write(dir.path(), path, text);
    }

    let plant = scan(dir.path()).expect("scan the small app");

    assert_eq!(
        machine_names(&plant),
        [
            "auth",
            "dashboard",
            "errors",
            "links",
            "middleware",
            "ratelimit",
            "server",
            "store",
            "test",
        ],
    );
    let belts = belts(&plant);
    for belt in [
        "middleware -> auth x1",
        "server -> middleware x1",
        "test -> links x1",
        "test -> server x1",
    ] {
        assert!(belts.iter().any(|b| b == belt), "missing belt {belt:?} in {belts:?}");
    }
}

#[test]
fn a_repo_over_forty_files_groups_by_directory() {
    let dir = tempfile::tempdir().expect("temp dir");
    for i in 0..41 {
        write(dir.path(), &format!("src/feature/f{i}.ts"), "export const x = 1;\n");
    }

    let plant = scan(dir.path()).expect("scan the large app");

    assert_eq!(machine_names(&plant), ["src/feature"]);
}

#[test]
fn module_names_that_clash_fall_back_to_their_paths() {
    let dir = tempfile::tempdir().expect("temp dir");
    for name in ["a", "b", "c", "d", "test"] {
        write(dir.path(), &format!("src/{name}.ts"), "export const x = 1;\n");
    }
    write(dir.path(), "test/only.ts", "export const y = 1;\n");

    let plant = scan(dir.path()).expect("scan the clashing app");

    assert_eq!(
        machine_names(&plant),
        ["a", "b", "c", "d", "src/test", "test"],
        "src/test.ts cannot take the name of the test directory",
    );
}

#[test]
fn two_modules_with_one_stem_keep_their_paths_apart() {
    let dir = tempfile::tempdir().expect("temp dir");
    for name in ["a", "b", "c", "d", "index"] {
        write(dir.path(), &format!("src/{name}.ts"), "export const x = 1;\n");
    }
    for name in ["e", "f", "g", "h", "index"] {
        write(dir.path(), &format!("src/ui/{name}.ts"), "export const x = 1;\n");
    }

    let plant = scan(dir.path()).expect("scan the app with two index files");

    assert_eq!(
        machine_names(&plant),
        ["a", "b", "c", "d", "e", "f", "g", "h", "src/index", "src/ui/index"],
    );
}
