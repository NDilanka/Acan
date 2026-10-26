use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{Belt, FileId, ImportTarget, Machine, MachineId, Plant, SourceFile};

/// Assigns files to machines by name. Files not listed here group by their directory.
///
/// The foreman edits this data and calls [`Plant::regroup`]; the file list and symbols stay the same.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grouping {
    /// Repo-relative file path to machine name.
    #[serde(default)]
    pub assign: BTreeMap<String, String>,
}

impl Plant {
    /// Recomputes machines and belts from the files under `grouping`.
    pub fn regroup(&mut self, grouping: &Grouping) {
        let (machines, belts) = build(&self.files, grouping);
        self.machines = machines;
        self.belts = belts;
    }
}

fn build(files: &[SourceFile], grouping: &Grouping) -> (Vec<Machine>, Vec<Belt>) {
    let mut groups: BTreeMap<String, Vec<&SourceFile>> = BTreeMap::new();
    for file in files {
        let name = grouping
            .assign
            .get(&file.path)
            .cloned()
            .unwrap_or_else(|| directory_of(&file.path).to_string());
        groups.entry(name).or_default().push(file);
    }

    let mut machine_of: BTreeMap<FileId, MachineId> = BTreeMap::new();
    let mut machines = Vec::with_capacity(groups.len());
    for (index, (name, members)) in groups.into_iter().enumerate() {
        let id = MachineId(index as u32);
        for file in &members {
            machine_of.insert(file.id, id);
        }
        let mut lang_mix = BTreeMap::new();
        for file in &members {
            *lang_mix.entry(file.lang).or_insert(0) += 1;
        }
        machines.push(Machine {
            id,
            name,
            dir: common_directory(members.iter().map(|file| file.path.as_str())),
            files: members.iter().map(|file| file.id).collect(),
            lang_mix,
        });
    }

    let mut counts: BTreeMap<(MachineId, MachineId), u32> = BTreeMap::new();
    for file in files {
        let Some(&from) = machine_of.get(&file.id) else {
            continue;
        };
        for import in &file.imports {
            if let ImportTarget::File { file: target } = import.target {
                let Some(&to) = machine_of.get(&target) else {
                    continue;
                };
                if to != from {
                    *counts.entry((from, to)).or_insert(0) += 1;
                }
            }
        }
    }
    let belts = counts
        .into_iter()
        .map(|((from, to), imports)| Belt { from, to, imports })
        .collect();
    (machines, belts)
}

fn directory_of(path: &str) -> &str {
    path.rsplit_once('/').map_or(".", |(dir, _)| dir)
}

fn common_directory<'a>(paths: impl Iterator<Item = &'a str>) -> String {
    let mut common: Option<Vec<&str>> = None;
    for path in paths {
        let dir: Vec<&str> = directory_of(path)
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect();
        common = Some(match common {
            None => dir,
            Some(previous) => previous
                .iter()
                .zip(&dir)
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| *a)
                .collect(),
        });
    }
    let parts = common.unwrap_or_default();
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}
