use std::fs;
use std::path::Path;
use std::str::FromStr;

use acan_core::{Belt, FilePath, MachineKey};
use acan_server::scan;

const STORE_TS: &str =
    "export function load() {\n  return 1;\n}\n\nfunction helper() {\n  return 2;\n}\n";
const VIEW_TS: &str =
    "import { load } from \"../model/store\";\n\nexport function render() {\n  return load();\n}\n";

fn write_fixture(root: &Path) {
    fs::create_dir_all(root.join("model")).expect("model dir");
    fs::create_dir_all(root.join("ui")).expect("ui dir");
    fs::write(root.join("model/store.ts"), STORE_TS).expect("write store");
    fs::write(root.join("ui/view.ts"), VIEW_TS).expect("write view");
}

fn path(raw: &str) -> FilePath {
    FilePath::from_str(raw).expect("valid repo path")
}

fn key(raw: &str) -> MachineKey {
    MachineKey::from_str(raw).expect("valid machine key")
}

#[test]
fn index_keeps_paths_machines_symbol_ranges_exports_and_belts() {
    let dir = tempfile::tempdir().expect("temp dir");
    write_fixture(dir.path());

    let scanned = scan(dir.path()).expect("scan fixture");
    let index = &scanned.index;

    let files: Vec<&str> = index.files.keys().map(FilePath::as_str).collect();
    assert_eq!(
        files,
        ["model/store.ts", "ui/view.ts"],
        "every source file is indexed"
    );

    let view = &index.files[&path("ui/view.ts")];
    assert_eq!(
        view.machine,
        key("ui"),
        "a file belongs to the machine of its directory"
    );

    let render = view
        .symbols
        .iter()
        .find(|symbol| symbol.name == "render")
        .expect("render is indexed");
    assert!(render.exported, "an exported function is marked exported");
    let rendered = &VIEW_TS[render.bytes.start as usize..render.bytes.end as usize];
    assert!(
        rendered.contains("function render"),
        "the byte range covers the symbol's source, got {rendered:?}"
    );

    let store = &index.files[&path("model/store.ts")];
    let helper = store
        .symbols
        .iter()
        .find(|symbol| symbol.name == "helper")
        .expect("helper is indexed");
    assert!(
        !helper.exported,
        "a function without export is not exported"
    );

    let belt = Belt {
        from: key("ui"),
        to: key("model"),
    };
    assert!(
        index.belts.contains(&belt),
        "ui imports model, so ui -> model is a belt"
    );
    assert_eq!(index.belts.len(), 1, "no other belts exist in this fixture");

    assert_eq!(
        index.rev.as_str(),
        scanned.plant.revision.to_string(),
        "the index revision is the plant revision"
    );
}

#[test]
fn equal_contents_give_equal_revisions_and_an_edit_changes_it() {
    let dir = tempfile::tempdir().expect("temp dir");
    write_fixture(dir.path());
    let first = scan(dir.path()).expect("first scan").index.rev;
    let second = scan(dir.path()).expect("second scan").index.rev;
    assert_eq!(first, second, "an unchanged repo keeps its revision");

    fs::write(
        dir.path().join("ui/view.ts"),
        format!("{VIEW_TS}// edited\n"),
    )
    .expect("edit");
    let edited = scan(dir.path()).expect("scan after edit").index.rev;
    assert_ne!(first, edited, "an edited file moves the revision");
}
