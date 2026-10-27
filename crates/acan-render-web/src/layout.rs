//! Places the plant's machines on a plan view and routes their belts.
//!
//! Machines are placed in a grid, in the order a walk over the belts reaches them, so machines
//! that import each other sit next to each other and their belts stay short. Positions are world
//! units; the camera turns them into pixels.

use std::collections::{BTreeMap, VecDeque};

use acan_core::Locus;
use serde::Deserialize;

/// Width of a machine footprint, in world units.
const MACHINE_W: f32 = 176.0;
/// Height of a machine footprint, in world units.
const MACHINE_H: f32 = 88.0;
/// Space between footprints in a row. Belts run through it.
const GAP_X: f32 = 120.0;
/// Space between footprints in a column.
const GAP_Y: f32 = 110.0;
/// Columns per square root of the machine count. Above 1 gives a wider than tall plan.
const COLUMNS_PER_SQRT: f32 = 1.5;

/// The plant as `GET /api/plant` returns it. Only the fields the plan reads are named.
#[derive(Debug, Clone, Deserialize)]
pub struct PlantJson {
    /// Machines with their files.
    pub machines: Vec<MachineJson>,
    /// Import edges between machines.
    pub belts: Vec<BeltJson>,
    /// Every scanned file, with the id the machines refer to it by.
    pub files: Vec<FileJson>,
}

/// One machine of the plant.
#[derive(Debug, Clone, Deserialize)]
pub struct MachineJson {
    /// The machine id the belts refer to.
    pub id: u32,
    /// The machine name, which is also its machine key in the origin.
    pub name: String,
    /// The files the machine groups.
    pub files: Vec<u32>,
}

/// An import edge between two machines.
#[derive(Debug, Clone, Deserialize)]
pub struct BeltJson {
    /// The importing machine id.
    pub from: u32,
    /// The imported machine id.
    pub to: u32,
}

/// A scanned file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileJson {
    /// The file id the machines refer to it by.
    pub id: u32,
    /// The repo-relative path, which is also its file path in the origin.
    pub path: String,
}

/// One machine's footprint on the plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanMachine {
    /// The machine name.
    pub name: String,
    /// The top-left corner, in world units.
    pub min: [f32; 2],
    /// The width and height, in world units.
    pub size: [f32; 2],
}

impl PlanMachine {
    /// The middle of the footprint.
    #[must_use]
    pub fn center(&self) -> [f32; 2] {
        [
            self.min[0] + self.size[0] * 0.5,
            self.min[1] + self.size[1] * 0.5,
        ]
    }
}

/// A belt drawn as a polyline through the centers of its two footprints.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBelt {
    /// Index of the importing machine in [`Plan::machines`].
    pub from: usize,
    /// Index of the imported machine in [`Plan::machines`].
    pub to: usize,
    /// The corners of the route, in world units.
    pub points: Vec<[f32; 2]>,
}

/// The plan view of one plant: where each machine and belt is drawn.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Plan {
    /// Machines, sorted by name.
    pub machines: Vec<PlanMachine>,
    /// Belts between them.
    pub belts: Vec<PlanBelt>,
    by_name: BTreeMap<String, usize>,
    by_path: BTreeMap<String, usize>,
}

impl Plan {
    /// Lays out `plant`. The same plant always gives the same plan.
    #[must_use]
    pub fn new(plant: &PlantJson) -> Self {
        let mut sorted: Vec<&MachineJson> = plant.machines.iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        let index_of_id: BTreeMap<u32, usize> =
            sorted.iter().enumerate().map(|(i, m)| (m.id, i)).collect();
        let path_of_id: BTreeMap<u32, &str> = plant
            .files
            .iter()
            .map(|f| (f.id, f.path.as_str()))
            .collect();
        let mut by_path = BTreeMap::new();
        for (i, machine) in sorted.iter().enumerate() {
            for file in &machine.files {
                if let Some(path) = path_of_id.get(file) {
                    by_path.insert((*path).to_owned(), i);
                }
            }
        }

        let links: Vec<(usize, usize)> = plant
            .belts
            .iter()
            .filter_map(|belt| {
                let from = *index_of_id.get(&belt.from)?;
                let to = *index_of_id.get(&belt.to)?;
                (from != to).then_some((from, to))
            })
            .collect();
        let order = placement_order(sorted.len(), &links);

        let columns = columns_for(sorted.len());
        let mut machines: Vec<PlanMachine> = sorted
            .iter()
            .map(|m| PlanMachine {
                name: m.name.clone(),
                min: [0.0, 0.0],
                size: [MACHINE_W, MACHINE_H],
            })
            .collect();
        for (slot, &i) in order.iter().enumerate() {
            let (col, row) = (slot % columns, slot / columns);
            machines[i].min = [
                col as f32 * (MACHINE_W + GAP_X),
                row as f32 * (MACHINE_H + GAP_Y),
            ];
        }

        let belts = links
            .iter()
            .map(|&(from, to)| {
                let (a, b) = (machines[from].center(), machines[to].center());
                PlanBelt {
                    from,
                    to,
                    points: vec![a, [b[0], a[1]], b],
                }
            })
            .collect();
        let by_name = machines
            .iter()
            .enumerate()
            .map(|(i, m)| (m.name.clone(), i))
            .collect();

        Self {
            machines,
            belts,
            by_name,
            by_path,
        }
    }

    /// The index of the machine called `name`.
    #[must_use]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.by_name.get(name).copied()
    }

    /// The index of the machine that groups the file at `path`.
    #[must_use]
    pub fn machine_of_path(&self, path: &str) -> Option<usize> {
        self.by_path.get(path).copied()
    }

    /// The machines a place on the plant touches. A belt touches both of its ends.
    #[must_use]
    pub fn machines_at(&self, locus: &Locus) -> Vec<usize> {
        match locus {
            Locus::Machine { machine } => self.index_of(machine.as_str()).into_iter().collect(),
            Locus::Belt { from, to } => [from.as_str(), to.as_str()]
                .into_iter()
                .filter_map(|name| self.index_of(name))
                .collect(),
            Locus::File { file } | Locus::Symbol { file, .. } => {
                self.machine_of_path(file.as_str()).into_iter().collect()
            }
        }
    }

    /// The point a place is drawn at: the middle of the machines it touches.
    #[must_use]
    pub fn point_at(&self, locus: &Locus) -> Option<[f32; 2]> {
        let centers: Vec<[f32; 2]> = self
            .machines_at(locus)
            .into_iter()
            .map(|i| self.machines[i].center())
            .collect();
        if centers.is_empty() {
            return None;
        }
        let n = centers.len() as f32;
        let sum = centers
            .iter()
            .fold([0.0, 0.0], |acc, c| [acc[0] + c[0], acc[1] + c[1]]);
        Some([sum[0] / n, sum[1] / n])
    }

    /// The box holding every footprint, as `(min, max)`, or `None` for an empty plant.
    #[must_use]
    pub fn bounds(&self) -> Option<([f32; 2], [f32; 2])> {
        self.machines.iter().fold(None, |acc, m| {
            let max = [m.min[0] + m.size[0], m.min[1] + m.size[1]];
            Some(match acc {
                None => (m.min, max),
                Some((lo, hi)) => (
                    [lo[0].min(m.min[0]), lo[1].min(m.min[1])],
                    [hi[0].max(max[0]), hi[1].max(max[1])],
                ),
            })
        })
    }
}

fn columns_for(machines: usize) -> usize {
    ((machines as f32).sqrt() * COLUMNS_PER_SQRT)
        .ceil()
        .max(1.0) as usize
}

/// Machines in breadth-first order over the belts, each connected group in turn.
fn placement_order(n: usize, links: &[(usize, usize)]) -> Vec<usize> {
    let mut adjacent = vec![Vec::new(); n];
    for &(a, b) in links {
        adjacent[a].push(b);
        adjacent[b].push(a);
    }
    let mut seen = vec![false; n];
    let mut order = Vec::with_capacity(n);
    for start in 0..n {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut queue = VecDeque::from([start]);
        while let Some(i) = queue.pop_front() {
            order.push(i);
            for &j in &adjacent[i] {
                if !seen[j] {
                    seen[j] = true;
                    queue.push_back(j);
                }
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use acan_core::{FilePath, MachineKey};

    use super::*;

    fn machine(id: u32, name: &str, files: &[u32]) -> MachineJson {
        MachineJson {
            id,
            name: name.to_owned(),
            files: files.to_vec(),
        }
    }

    fn sample() -> PlantJson {
        PlantJson {
            machines: vec![
                machine(1, "api", &[10]),
                machine(2, "ui", &[11]),
                machine(3, "store", &[12]),
            ],
            belts: vec![BeltJson { from: 2, to: 1 }],
            files: vec![
                FileJson {
                    id: 10,
                    path: "src/api/handler.ts".to_owned(),
                },
                FileJson {
                    id: 11,
                    path: "src/ui/greet.ts".to_owned(),
                },
                FileJson {
                    id: 12,
                    path: "src/store.ts".to_owned(),
                },
            ],
        }
    }

    #[test]
    fn machines_are_sorted_and_indexed_by_name_and_path() {
        let plan = Plan::new(&sample());
        let names: Vec<&str> = plan.machines.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, ["api", "store", "ui"]);
        assert_eq!(plan.index_of("ui"), Some(2));
        assert_eq!(plan.machine_of_path("src/api/handler.ts"), Some(0));
        assert_eq!(plan.machine_of_path("missing.ts"), None);
    }

    #[test]
    fn connected_machines_are_placed_next_to_each_other() {
        // ui imports api, so the walk visits api first and ui right after it.
        let order = placement_order(3, &[(2, 0)]);
        assert_eq!(order, [0, 2, 1]);
    }

    #[test]
    fn a_belt_runs_between_the_centers_of_its_machines() {
        let plan = Plan::new(&sample());
        let belt = &plan.belts[0];
        assert_eq!(
            belt.points.first(),
            Some(&plan.machines[belt.from].center())
        );
        assert_eq!(belt.points.last(), Some(&plan.machines[belt.to].center()));
    }

    #[test]
    fn a_locus_maps_to_the_machines_it_names() {
        let plan = Plan::new(&sample());
        let symbol = Locus::Symbol {
            file: FilePath::from_str("src/api/handler.ts").expect("valid path"),
            name: "handle".to_owned(),
        };
        assert_eq!(plan.machines_at(&symbol), [0]);
        let belt = Locus::Belt {
            from: MachineKey::from_str("ui").expect("valid key"),
            to: MachineKey::from_str("api").expect("valid key"),
        };
        assert_eq!(plan.machines_at(&belt), [2, 0]);
        let midpoint = plan.point_at(&belt).expect("both ends are on the plan");
        let (a, b) = (plan.machines[2].center(), plan.machines[0].center());
        assert_eq!(midpoint, [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]);
    }

    #[test]
    fn bounds_cover_every_footprint() {
        let plan = Plan::new(&sample());
        let (lo, hi) = plan.bounds().expect("a plan with machines has bounds");
        for m in &plan.machines {
            assert!(m.min[0] >= lo[0] && m.min[1] >= lo[1]);
            assert!(m.min[0] + m.size[0] <= hi[0] && m.min[1] + m.size[1] <= hi[1]);
        }
        assert_eq!(Plan::default().bounds(), None);
    }

    #[test]
    fn the_same_plant_gives_the_same_plan() {
        assert_eq!(Plan::new(&sample()), Plan::new(&sample()));
    }
}
