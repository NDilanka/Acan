//! Plant layout on the ground plan. Machines are layered by import direction (longest path),
//! ordered inside each layer by the barycentre of their neighbours, and stacked on a grid.
//! A belt that spans several layers crosses each one in a lane of its own, so every belt
//! runs through a gap between columns or through a lane, never through a footprint. The
//! result depends only on the input, so the same plant always lays out the same way.

use std::collections::VecDeque;

use crate::kind::Kind;

/// Positions snap to this grid, in plan units.
pub const GRID: f32 = 8.0;
/// Clear width between columns. Belts run through it.
pub const COLUMN_GAP: f32 = 72.0;
/// Clear height between items stacked in one column.
pub const ROW_GAP: f32 = 40.0;
/// Height of the band a belt crosses a column in when it passes that column without a machine.
pub const LANE: f32 = 16.0;
/// Horizontal step between belts that share a gap: the widest belt plus a clear gap.
const BELT_STEP: f32 = 8.0;
/// Belts in one gap cycle through this many offsets, which keeps every lane clear of the
/// columns on either side.
const BELT_SLOTS: usize = 7;
/// Barycentre sweeps. Each sweep moves down or up one layer at a time.
const SWEEPS: usize = 8;

/// A directed import between two machines, by index. `from` imports `to`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub imports: u32,
}

/// A machine on the plan. Its footprint is `kind.footprint()`, with the top left at `(x, y)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placed {
    pub x: f32,
    pub y: f32,
    pub kind: Kind,
}

impl Placed {
    /// `[min x, min y, max x, max y]` of the footprint.
    pub fn footprint(&self) -> [f32; 4] {
        let (w, d) = self.kind.footprint();
        [self.x, self.y, self.x + w, self.y + d]
    }
}

/// An orthogonal belt, from the edge of one plinth to the edge of another. `path` is the
/// centre line before its corners are rounded.
#[derive(Clone, Debug, PartialEq)]
pub struct Belt {
    pub from: usize,
    pub to: usize,
    pub imports: u32,
    pub path: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    /// One entry per input machine, in input order.
    pub machines: Vec<Placed>,
    /// One entry per routed edge, in input order. Edges that closed a cycle are not routed.
    pub belts: Vec<Belt>,
    /// `[min x, min y, max x, max y]` of all footprints.
    pub bounds: [f32; 4],
}

pub fn layout(kinds: &[Kind], edges: &[Edge]) -> Layout {
    let machines = kinds.len();
    let accepted = acyclic_edges(machines, edges);
    let layer = longest_path_layers(machines, edges, &accepted);
    let graph = Graph::build(&layer, edges, &accepted);
    let rows = order_rows(&graph);

    let node_height = |node: usize| -> f32 {
        if node < machines {
            snap_up(kinds[node].footprint().1)
        } else {
            LANE
        }
    };
    let widths: Vec<f32> = rows
        .iter()
        .map(|row| {
            row.iter()
                .filter(|&&node| node < machines)
                .map(|&node| snap_up(kinds[node].footprint().0))
                .fold(0.0, f32::max)
        })
        .collect();
    let mut column_x = Vec::with_capacity(rows.len());
    let mut x = 0.0;
    for width in &widths {
        column_x.push(x);
        x += width + COLUMN_GAP;
    }

    let stacks: Vec<f32> = rows
        .iter()
        .map(|row| {
            let items: f32 = row.iter().map(|&node| node_height(node)).sum();
            items + ROW_GAP * row.len().saturating_sub(1) as f32
        })
        .collect();
    let tallest = stacks.iter().copied().fold(0.0, f32::max);
    let mut top = vec![0.0_f32; graph.layer.len()];
    for (l, row) in rows.iter().enumerate() {
        let mut y = snap((tallest - stacks[l]) / 2.0);
        for &node in row {
            top[node] = y;
            y += node_height(node) + ROW_GAP;
        }
    }

    let mut placed = Vec::with_capacity(machines);
    for (node, &kind) in kinds.iter().enumerate() {
        let l = graph.layer[node];
        let (w, _) = kind.footprint();
        placed.push(Placed {
            x: column_x[l] + snap((widths[l] - w) / 2.0),
            y: top[node],
            kind,
        });
    }

    let mut out_total = vec![0_usize; machines];
    let mut in_total = vec![0_usize; machines];
    for chain in &graph.chains {
        out_total[chain[0]] += 1;
        in_total[chain[chain.len() - 1]] += 1;
    }
    let mut out_seen = vec![0_usize; machines];
    let mut in_seen = vec![0_usize; machines];
    let mut exit_y = Vec::with_capacity(graph.chains.len());
    let mut entry_y = Vec::with_capacity(graph.chains.len());
    for chain in &graph.chains {
        let (from, to) = (chain[0], chain[chain.len() - 1]);
        let j = out_seen[from];
        out_seen[from] += 1;
        exit_y.push(spread(placed[from], j, out_total[from]));
        let j = in_seen[to];
        in_seen[to] += 1;
        entry_y.push(spread(placed[to], j, in_total[to]));
    }

    let mut gap_count = vec![0_usize; rows.len()];
    let mut belts = Vec::with_capacity(graph.chains.len());
    for (k, chain) in graph.chains.iter().enumerate() {
        let from = placed[chain[0]];
        let to = placed[chain[chain.len() - 1]];
        let mut y = exit_y[k];
        let mut path = vec![[from.x + from.kind.footprint().0, y]];
        for &node in &chain[1..] {
            let l = graph.layer[node];
            let slot = gap_count[l];
            gap_count[l] += 1;
            let offset = (slot % BELT_SLOTS) as f32 - (BELT_SLOTS / 2) as f32;
            let jog_x = column_x[l] - COLUMN_GAP / 2.0 + offset * BELT_STEP;
            path.push([jog_x, y]);
            y = if node < machines {
                entry_y[k]
            } else {
                top[node] + LANE / 2.0
            };
            path.push([jog_x, y]);
        }
        path.push([to.x, y]);
        path.dedup();
        let edge = edges[accepted[k]];
        belts.push(Belt {
            from: edge.from,
            to: edge.to,
            imports: edge.imports,
            path,
        });
    }

    let bounds = placed
        .iter()
        .map(Placed::footprint)
        .fold(None, |acc: Option<[f32; 4]>, f| match acc {
            None => Some(f),
            Some(b) => Some([
                b[0].min(f[0]),
                b[1].min(f[1]),
                b[2].max(f[2]),
                b[3].max(f[3]),
            ]),
        });
    Layout {
        machines: placed,
        belts,
        bounds: bounds.unwrap_or([0.0; 4]),
    }
}

/// Indices of the edges that keep the import graph acyclic, in input order. A self import, an
/// edge to a machine that does not exist, and an edge that closes a cycle are dropped.
fn acyclic_edges(machines: usize, edges: &[Edge]) -> Vec<usize> {
    let mut out = vec![Vec::new(); machines];
    let mut accepted = Vec::new();
    for (k, edge) in edges.iter().enumerate() {
        if edge.from >= machines
            || edge.to >= machines
            || edge.from == edge.to
            || reaches(&out, edge.to, edge.from)
        {
            continue;
        }
        out[edge.from].push(edge.to);
        accepted.push(k);
    }
    accepted
}

fn reaches(out: &[Vec<usize>], from: usize, to: usize) -> bool {
    let mut seen = vec![false; out.len()];
    let mut stack = vec![from];
    while let Some(node) = stack.pop() {
        if node == to {
            return true;
        }
        if std::mem::replace(&mut seen[node], true) {
            continue;
        }
        stack.extend(out[node].iter().copied());
    }
    false
}

/// Layer of each machine: the longest path from a source, so every edge points down a layer.
fn longest_path_layers(machines: usize, edges: &[Edge], accepted: &[usize]) -> Vec<usize> {
    let mut out = vec![Vec::new(); machines];
    let mut indegree = vec![0_usize; machines];
    for &k in accepted {
        let edge = edges[k];
        out[edge.from].push(edge.to);
        indegree[edge.to] += 1;
    }
    let mut layer = vec![0_usize; machines];
    let mut queue: VecDeque<usize> = (0..machines).filter(|&m| indegree[m] == 0).collect();
    while let Some(u) = queue.pop_front() {
        for &v in &out[u] {
            layer[v] = layer[v].max(layer[u] + 1);
            indegree[v] -= 1;
            if indegree[v] == 0 {
                queue.push_back(v);
            }
        }
    }
    layer
}

/// The layered graph: machines are nodes `0..machines`, and every layer a long edge crosses
/// adds one dummy node after them.
struct Graph {
    layer: Vec<usize>,
    up: Vec<Vec<usize>>,
    down: Vec<Vec<usize>>,
    /// Per accepted edge, its nodes from source to target.
    chains: Vec<Vec<usize>>,
}

impl Graph {
    fn build(machine_layer: &[usize], edges: &[Edge], accepted: &[usize]) -> Self {
        let mut layer = machine_layer.to_vec();
        let mut chains = Vec::with_capacity(accepted.len());
        for &k in accepted {
            let edge = edges[k];
            let mut chain = vec![edge.from];
            for l in machine_layer[edge.from] + 1..machine_layer[edge.to] {
                chain.push(layer.len());
                layer.push(l);
            }
            chain.push(edge.to);
            chains.push(chain);
        }
        let mut up = vec![Vec::new(); layer.len()];
        let mut down = vec![Vec::new(); layer.len()];
        for chain in &chains {
            for pair in chain.windows(2) {
                down[pair[0]].push(pair[1]);
                up[pair[1]].push(pair[0]);
            }
        }
        Self {
            layer,
            up,
            down,
            chains,
        }
    }
}

/// Nodes of each layer in drawing order, from the barycentre sweeps.
fn order_rows(graph: &Graph) -> Vec<Vec<usize>> {
    let layers = graph.layer.iter().copied().max().map_or(0, |max| max + 1);
    let mut rows = vec![Vec::new(); layers];
    for (node, &l) in graph.layer.iter().enumerate() {
        rows[l].push(node);
    }
    let mut pos = vec![0_usize; graph.layer.len()];
    for row in &rows {
        set_positions(row, &mut pos);
    }
    for sweep in 0..SWEEPS {
        let downward = sweep % 2 == 0;
        let order: Vec<usize> = if downward {
            (1..layers).collect()
        } else {
            (0..layers.saturating_sub(1)).rev().collect()
        };
        let neighbours = if downward { &graph.up } else { &graph.down };
        for l in order {
            let mut keyed: Vec<(f64, usize, usize)> = rows[l]
                .iter()
                .enumerate()
                .map(|(i, &node)| {
                    let adjacent = &neighbours[node];
                    let key = if adjacent.is_empty() {
                        i as f64
                    } else {
                        let sum: f64 = adjacent.iter().map(|&a| pos[a] as f64).sum();
                        sum / adjacent.len() as f64
                    };
                    (key, i, node)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            rows[l] = keyed.into_iter().map(|(_, _, node)| node).collect();
            set_positions(&rows[l], &mut pos);
        }
    }
    rows
}

fn set_positions(row: &[usize], pos: &mut [usize]) {
    for (i, &node) in row.iter().enumerate() {
        pos[node] = i;
    }
}

/// The point on a plinth's right or left edge where belt `index` of `count` leaves or
/// enters. Belts spread evenly over the edge's depth.
fn spread(machine: Placed, index: usize, count: usize) -> f32 {
    let (_, d) = machine.kind.footprint();
    machine.y + d * (index + 1) as f32 / (count + 1) as f32
}

fn snap(value: f32) -> f32 {
    (value / GRID).round() * GRID
}

fn snap_up(value: f32) -> f32 {
    (value / GRID).ceil() * GRID
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plant::{Plant, from_scan_json, stress};

    const FIXTURE: &str = include_str!("../www/fixtures/demo-plant.json");

    fn chain_plant() -> (Vec<Kind>, Vec<Edge>) {
        let kinds = vec![
            Kind::Gate,
            Kind::Assembler,
            Kind::Tank,
            Kind::Crate,
            Kind::Assembler,
        ];
        let edges = vec![
            Edge {
                from: 0,
                to: 1,
                imports: 1,
            },
            Edge {
                from: 1,
                to: 2,
                imports: 2,
            },
            Edge {
                from: 0,
                to: 2,
                imports: 1,
            },
            Edge {
                from: 3,
                to: 4,
                imports: 1,
            },
            Edge {
                from: 2,
                to: 0,
                imports: 1,
            },
        ];
        (kinds, edges)
    }

    fn inside(footprint: [f32; 4], p: [f32; 2]) -> bool {
        let inset = 0.5;
        p[0] > footprint[0] + inset
            && p[0] < footprint[2] - inset
            && p[1] > footprint[1] + inset
            && p[1] < footprint[3] - inset
    }

    fn on_boundary(footprint: [f32; 4], p: [f32; 2]) -> bool {
        let eps = 1e-3;
        let within = p[0] >= footprint[0] - eps
            && p[0] <= footprint[2] + eps
            && p[1] >= footprint[1] - eps
            && p[1] <= footprint[3] + eps;
        within
            && ((p[0] - footprint[0]).abs() < eps
                || (p[0] - footprint[2]).abs() < eps
                || (p[1] - footprint[1]).abs() < eps
                || (p[1] - footprint[3]).abs() < eps)
    }

    /// Checks every invariant the brief names on one laid-out plant.
    fn check_invariants(layout: &Layout) {
        let boxes: Vec<[f32; 4]> = layout.machines.iter().map(Placed::footprint).collect();
        for (i, a) in boxes.iter().enumerate() {
            for (j, b) in boxes.iter().enumerate().skip(i + 1) {
                let apart = a[2] + GRID <= b[0]
                    || b[2] + GRID <= a[0]
                    || a[3] + GRID <= b[1]
                    || b[3] + GRID <= a[1];
                assert!(
                    apart,
                    "footprints {i} and {j} overlap or touch: {a:?} {b:?}"
                );
            }
        }
        for belt in &layout.belts {
            let path = &belt.path;
            assert!(
                path.len() >= 2,
                "belt {}->{} has no segment",
                belt.from,
                belt.to
            );
            for pair in path.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let horizontal = (a[1] - b[1]).abs() < 1e-3;
                let vertical = (a[0] - b[0]).abs() < 1e-3;
                assert!(
                    horizontal ^ vertical,
                    "belt {}->{} has a diagonal or empty segment: {a:?} {b:?}",
                    belt.from,
                    belt.to
                );
                let steps = ((b[0] - a[0]).abs() + (b[1] - a[1]).abs()).ceil() as usize;
                for s in 0..=steps {
                    let t = s as f32 / steps.max(1) as f32;
                    let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
                    for (m, footprint) in boxes.iter().enumerate() {
                        assert!(
                            !inside(*footprint, p),
                            "belt {}->{} runs under machine {m} at {p:?}",
                            belt.from,
                            belt.to
                        );
                    }
                }
            }
            let first = path[0];
            let last = path[path.len() - 1];
            assert!(
                on_boundary(boxes[belt.from], first),
                "belt {}->{} does not start on its plinth edge: {first:?}",
                belt.from,
                belt.to
            );
            assert!(
                on_boundary(boxes[belt.to], last),
                "belt {}->{} does not end on its plinth edge: {last:?}",
                belt.from,
                belt.to
            );
        }
    }

    fn check_plant(plant: &Plant) {
        let (kinds, edges) = plant.layout_input();
        let layout = layout(&kinds, &edges);
        assert_eq!(layout.machines.len(), plant.machines.len());
        check_invariants(&layout);
    }

    #[test]
    fn layout_is_deterministic() {
        let (kinds, edges) = chain_plant();
        assert_eq!(layout(&kinds, &edges), layout(&kinds, &edges));
        let plant = stress(300);
        let (kinds, edges) = plant.layout_input();
        assert_eq!(layout(&kinds, &edges), layout(&kinds, &edges));
    }

    #[test]
    fn imports_flow_left_to_right_by_layer() {
        let (kinds, edges) = chain_plant();
        let laid = layout(&kinds, &edges);
        let x = |m: usize| laid.machines[m].x;
        assert!(x(0) < x(1), "gate imports assembler, so it sits left of it");
        assert!(x(1) < x(2), "assembler imports tank, so it sits left of it");
    }

    #[test]
    fn a_closing_edge_is_dropped_not_routed() {
        let (kinds, edges) = chain_plant();
        let laid = layout(&kinds, &edges);
        assert_eq!(laid.belts.len(), 4);
        assert!(!laid.belts.iter().any(|b| b.from == 2 && b.to == 0));
    }

    #[test]
    fn a_two_node_cycle_routes_one_belt() {
        let kinds = [Kind::Gate, Kind::Tank];
        let edges = [
            Edge {
                from: 0,
                to: 1,
                imports: 1,
            },
            Edge {
                from: 1,
                to: 0,
                imports: 1,
            },
        ];
        let laid = layout(&kinds, &edges);
        assert_eq!(laid.belts.len(), 1);
        check_invariants(&laid);
    }

    #[test]
    #[should_panic(expected = "runs under machine")]
    fn the_checker_rejects_a_belt_that_runs_under_a_machine() {
        let laid = Layout {
            machines: vec![
                Placed {
                    x: 0.0,
                    y: 0.0,
                    kind: Kind::Assembler,
                },
                Placed {
                    x: 200.0,
                    y: 0.0,
                    kind: Kind::Assembler,
                },
                Placed {
                    x: 400.0,
                    y: 0.0,
                    kind: Kind::Assembler,
                },
            ],
            belts: vec![Belt {
                from: 0,
                to: 2,
                imports: 1,
                path: vec![[96.0, 40.0], [400.0, 40.0]],
            }],
            bounds: [0.0, 0.0, 496.0, 86.0],
        };
        check_invariants(&laid);
    }

    #[test]
    fn an_empty_plant_has_an_empty_layout() {
        let laid = layout(&[], &[]);
        assert!(laid.machines.is_empty());
        assert!(laid.belts.is_empty());
        assert_eq!(laid.bounds, [0.0; 4]);
    }

    #[test]
    fn a_hand_built_plant_meets_every_invariant() {
        let (kinds, edges) = chain_plant();
        check_invariants(&layout(&kinds, &edges));
    }

    #[test]
    fn the_demo_fixture_meets_every_invariant() {
        let plant = from_scan_json(FIXTURE).expect("fixture parses");
        check_plant(&plant);
    }

    #[test]
    fn a_large_generated_plant_meets_every_invariant() {
        check_plant(&stress(400));
    }

    #[test]
    fn bounds_cover_every_footprint() {
        let plant = from_scan_json(FIXTURE).expect("fixture parses");
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        for placed in &laid.machines {
            let f = placed.footprint();
            assert!(f[0] >= laid.bounds[0] && f[2] <= laid.bounds[2]);
            assert!(f[1] >= laid.bounds[1] && f[3] <= laid.bounds[3]);
        }
    }
}
