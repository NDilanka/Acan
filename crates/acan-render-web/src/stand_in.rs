//! Stand-in machine models, the floor, and the belt channels, until acan-iso supplies real
//! `Figure`s. A model is a plinth slab with a box body on it, in four tones with outlines,
//! and it is tagged with all three detail levels. Everything is projected with the renderer's
//! own projection, so the plan layout and the drawings agree.

use acan_render::{Detail, DrawList, Fill, Ink, Instance, Item, Scene, depth, project};

use crate::kind::Kind;
use crate::layout::{Belt, GRID, Layout, Placed};

const OUTLINE: f32 = 0.6;
const FLOOR_MARGIN: f32 = 64.0;
const FLOOR_THICKNESS: f32 = 12.0;
/// Radius of a belt's rounded corners, in plan units.
const BELT_CORNER: f32 = 10.0;
/// Segments per rounded corner.
const CORNER_STEPS: usize = 4;
const SCREW_RADIUS: f32 = 1.3;
const RING_SEGMENTS: usize = 24;

/// The parts that differ between kinds. The footprint comes from `Kind::footprint`.
struct Shape {
    plinth: f32,
    body: f32,
    ring: bool,
    screws: usize,
}

fn shape(kind: Kind) -> Shape {
    match kind {
        Kind::Intake => Shape {
            plinth: 6.0,
            body: 26.0,
            ring: false,
            screws: 2,
        },
        Kind::Gate => Shape {
            plinth: 6.0,
            body: 30.0,
            ring: true,
            screws: 4,
        },
        Kind::Tank => Shape {
            plinth: 6.0,
            body: 44.0,
            ring: true,
            screws: 4,
        },
        Kind::Splitter => Shape {
            plinth: 6.0,
            body: 22.0,
            ring: false,
            screws: 6,
        },
        Kind::Valve => Shape {
            plinth: 6.0,
            body: 28.0,
            ring: true,
            screws: 2,
        },
        Kind::Assembler => Shape {
            plinth: 6.0,
            body: 34.0,
            ring: false,
            screws: 4,
        },
        Kind::Crate => Shape {
            plinth: 4.0,
            body: 18.0,
            ring: false,
            screws: 2,
        },
    }
}

/// One model per kind, in `Kind::ALL` order. Each is in its own plan frame, with the plinth's
/// top left corner at the origin.
pub fn models() -> Vec<DrawList> {
    Kind::ALL.iter().map(|&kind| model(kind)).collect()
}

fn model(kind: Kind) -> DrawList {
    let (w, d) = kind.footprint();
    let s = shape(kind);
    let inset = if kind == Kind::Crate { 6.0 } else { 12.0 };
    let (p, top) = (s.plinth, s.plinth + s.body);
    let (x0, y0, x1, y1) = (inset, inset, w - inset, d - inset);
    let (cx, cy) = (w / 2.0, d / 2.0);
    let mut items = Vec::new();

    let plinth_top = [[0.0, 0.0, p], [w, 0.0, p], [w, d, p], [0.0, d, p]];
    push_fill(
        &mut items,
        [[w, 0.0, 0.0], [w, d, 0.0], [w, d, p], [w, 0.0, p]],
        Fill::Shade1,
    );
    push_fill(
        &mut items,
        [[0.0, d, 0.0], [w, d, 0.0], [w, d, p], [0.0, d, p]],
        Fill::Shade0,
    );
    push_fill(&mut items, plinth_top, Fill::Top);
    push_line(&mut items, &plinth_top, true, Ink::Lo, Detail::Always);
    push_line(
        &mut items,
        &[[w, d, 0.0], [w, d, p]],
        false,
        Ink::Mid,
        Detail::Always,
    );

    push_fill(
        &mut items,
        [[x1, y0, p], [x1, y1, p], [x1, y1, top], [x1, y0, top]],
        Fill::Shade2,
    );
    push_fill(
        &mut items,
        [[x0, y1, p], [x1, y1, p], [x1, y1, top], [x0, y1, top]],
        Fill::Shade1,
    );
    let body_top = [[x0, y0, top], [x1, y0, top], [x1, y1, top], [x0, y1, top]];
    push_fill(&mut items, body_top, Fill::LitTop);
    push_line(&mut items, &body_top, true, Ink::Hi, Detail::Always);
    push_line(
        &mut items,
        &[[x1, y1, p], [x1, y1, top]],
        false,
        Ink::Mid,
        Detail::Always,
    );

    let mid = p + s.body / 2.0;
    push_line(
        &mut items,
        &[[x0, y1, mid], [x1, y1, mid]],
        false,
        Ink::Lo,
        Detail::Near,
    );
    if s.ring {
        let r = (x1 - x0).min(y1 - y0) * 0.28;
        let ring: Vec<[f32; 3]> = (0..RING_SEGMENTS)
            .map(|k| {
                let a = std::f32::consts::TAU * k as f32 / RING_SEGMENTS as f32;
                [cx + r * a.cos(), cy + r * a.sin(), top]
            })
            .collect();
        push_line(&mut items, &ring, true, Ink::Mid, Detail::Near);
    }

    let inner = 6.0;
    let screw_sites = [
        [x0 + inner, y0 + inner],
        [x1 - inner, y0 + inner],
        [x1 - inner, y1 - inner],
        [x0 + inner, y1 - inner],
        [cx, y0 + inner],
        [x1 - inner, cy],
        [cx, y1 - inner],
        [x0 + inner, cy],
    ];
    let screws: Vec<[f32; 2]> = screw_sites
        .iter()
        .take(s.screws)
        .map(|&[x, y]| project(x, y, top))
        .collect();
    if !screws.is_empty() {
        items.push(Item::Dots {
            points: screws,
            radius: SCREW_RADIUS,
            tone: Ink::Mid,
            detail: Detail::Close,
        });
    }
    list(items)
}

/// The world drawn under the machines: the floor slab, then the belts on it.
pub fn scene(layout: &Layout) -> Scene {
    let floor = floor_rect(layout.bounds);
    let mut items = Vec::new();
    push_floor(&mut items, floor);
    for belt in &layout.belts {
        push_belt(&mut items, belt);
    }

    let mut order: Vec<&Placed> = layout.machines.iter().collect();
    order.sort_by(|a, b| machine_depth(a).total_cmp(&machine_depth(b)));
    let instances = order
        .into_iter()
        .map(|m| Instance {
            offset: project(m.x, m.y, 0.0),
            kind: m.kind.index() as u16,
            tint: 0,
        })
        .collect();
    Scene {
        world: list(items),
        floor_halo: Some(([floor[0], floor[1]], [floor[2], floor[3]])),
        instances,
    }
}

/// Painter depth of a machine's centre. Larger is nearer, so the scene draws ascending.
fn machine_depth(m: &Placed) -> f32 {
    let (w, d) = m.kind.footprint();
    depth(m.x + w / 2.0, m.y + d / 2.0, 0.0)
}

/// The floor rectangle `[min x, min y, max x, max y]`: the plant bounds, grown by the margin
/// and snapped outward to the grid.
fn floor_rect(bounds: [f32; 4]) -> [f32; 4] {
    [
        ((bounds[0] - FLOOR_MARGIN) / GRID).floor() * GRID,
        ((bounds[1] - FLOOR_MARGIN) / GRID).floor() * GRID,
        ((bounds[2] + FLOOR_MARGIN) / GRID).ceil() * GRID,
        ((bounds[3] + FLOOR_MARGIN) / GRID).ceil() * GRID,
    ]
}

fn push_floor(items: &mut Vec<Item>, [x0, y0, x1, y1]: [f32; 4]) {
    let t = FLOOR_THICKNESS;
    push_fill(
        items,
        [[x1, y0, 0.0], [x1, y1, 0.0], [x1, y1, -t], [x1, y0, -t]],
        Fill::Shade2,
    );
    push_fill(
        items,
        [[x0, y1, 0.0], [x1, y1, 0.0], [x1, y1, -t], [x0, y1, -t]],
        Fill::Shade0,
    );
    push_fill(
        items,
        [[x0, y0, 0.0], [x1, y0, 0.0], [x1, y1, 0.0], [x0, y1, 0.0]],
        Fill::Paper,
    );
}

/// A belt is a light channel with a fine edge line on each side.
fn push_belt(items: &mut Vec<Item>, belt: &Belt) {
    let centre = rounded(&belt.path);
    let half = belt_width(belt.imports) / 2.0;
    let normals = normals(&centre);
    let side = |sign: f32| -> Vec<[f32; 2]> {
        centre
            .iter()
            .zip(&normals)
            .map(|(p, n)| [p[0] + sign * n[0] * half, p[1] + sign * n[1] * half])
            .collect()
    };
    let left = side(1.0);
    let right = side(-1.0);
    let mut ring = left.clone();
    ring.extend(right.iter().rev().copied());
    items.push(Item::Fill {
        ring: project_all(&ring),
        tone: Fill::Shade3,
    });
    for edge in [left, right] {
        items.push(Item::Line {
            points: project_all(&edge),
            closed: false,
            tone: Ink::Faint,
            width: OUTLINE,
            detail: Detail::Always,
        });
    }
}

fn belt_width(imports: u32) -> f32 {
    8.0 + imports.min(8) as f32 * 0.5
}

/// Replaces each right-angle corner with a short quadratic curve. The curve starts and ends
/// on the two segments, at most halfway along each, so neighbouring corners never overlap.
fn rounded(path: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let Some((&first, rest)) = path.split_first() else {
        return Vec::new();
    };
    let Some(&last) = rest.last() else {
        return vec![first];
    };
    let mut out = vec![first];
    for window in path.windows(3) {
        let [a, p, b] = [window[0], window[1], window[2]];
        let (la, lb) = (distance(a, p), distance(p, b));
        let turn = cross(sub(p, a), sub(b, p));
        if turn.abs() < 1e-6 || la < 1e-6 || lb < 1e-6 {
            out.push(p);
            continue;
        }
        let r = BELT_CORNER.min(la / 2.0).min(lb / 2.0);
        let start = toward(p, a, r / la);
        let end = toward(p, b, r / lb);
        out.push(start);
        for k in 1..CORNER_STEPS {
            out.push(quadratic(start, p, end, k as f32 / CORNER_STEPS as f32));
        }
        out.push(end);
    }
    out.push(last);
    out
}

/// A unit normal at each point, from the directions of the segments on either side.
fn normals(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let direction = |a: [f32; 2], b: [f32; 2]| {
        let v = sub(b, a);
        let len = distance(a, b).max(1e-6);
        [v[0] / len, v[1] / len]
    };
    (0..points.len())
        .map(|i| {
            let incoming = (i > 0).then(|| direction(points[i - 1], points[i]));
            let outgoing = (i + 1 < points.len()).then(|| direction(points[i], points[i + 1]));
            let tangent = match (incoming, outgoing) {
                (Some(a), Some(b)) => {
                    let sum = [a[0] + b[0], a[1] + b[1]];
                    if sum[0].hypot(sum[1]) < 1e-6 { a } else { sum }
                }
                (Some(d), None) | (None, Some(d)) => d,
                (None, None) => [1.0, 0.0],
            };
            let len = tangent[0].hypot(tangent[1]).max(1e-6);
            [-tangent[1] / len, tangent[0] / len]
        })
        .collect()
}

fn quadratic(a: [f32; 2], c: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    let u = 1.0 - t;
    [
        u * u * a[0] + 2.0 * u * t * c[0] + t * t * b[0],
        u * u * a[1] + 2.0 * u * t * c[1] + t * t * b[1],
    ]
}

/// The point a fraction `t` of the way from `from` to `to`.
fn toward(from: [f32; 2], to: [f32; 2], t: f32) -> [f32; 2] {
    [
        from[0] + (to[0] - from[0]) * t,
        from[1] + (to[1] - from[1]) * t,
    ]
}

fn sub(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    let v = sub(b, a);
    v[0].hypot(v[1])
}

fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

fn project_all(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    points.iter().map(|p| project(p[0], p[1], 0.0)).collect()
}

fn push_fill(items: &mut Vec<Item>, corners: [[f32; 3]; 4], tone: Fill) {
    items.push(Item::Fill {
        ring: corners.iter().map(|p| project(p[0], p[1], p[2])).collect(),
        tone,
    });
}

fn push_line(items: &mut Vec<Item>, points: &[[f32; 3]], closed: bool, tone: Ink, detail: Detail) {
    items.push(Item::Line {
        points: points.iter().map(|p| project(p[0], p[1], p[2])).collect(),
        closed,
        tone,
        width: OUTLINE,
        detail,
    });
}

/// A draw list with its projected bounds filled in.
fn list(items: Vec<Item>) -> DrawList {
    let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    let mut grow = |p: &[f32; 2]| {
        bounds[0] = bounds[0].min(p[0]);
        bounds[1] = bounds[1].min(p[1]);
        bounds[2] = bounds[2].max(p[0]);
        bounds[3] = bounds[3].max(p[1]);
    };
    for item in &items {
        match item {
            Item::Fill { ring, .. } => ring.iter().for_each(&mut grow),
            Item::Line { points, .. } | Item::Dots { points, .. } => {
                points.iter().for_each(&mut grow)
            }
        }
    }
    if bounds[0] > bounds[2] {
        bounds = [0.0; 4];
    }
    DrawList { items, bounds }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Edge, layout};
    use crate::plant::stress;

    fn tags(list: &DrawList) -> (bool, bool, bool) {
        let mut seen = (false, false, false);
        for item in &list.items {
            let detail = match item {
                Item::Fill { .. } => Detail::Always,
                Item::Line { detail, .. } | Item::Dots { detail, .. } => *detail,
            };
            match detail {
                Detail::Always => seen.0 = true,
                Detail::Near => seen.1 = true,
                Detail::Close => seen.2 = true,
            }
        }
        seen
    }

    #[test]
    fn every_model_carries_all_three_detail_levels() {
        let models = models();
        assert_eq!(models.len(), Kind::ALL.len());
        for (kind, model) in Kind::ALL.iter().zip(&models) {
            assert_eq!(
                tags(model),
                (true, true, true),
                "{kind:?} lacks a detail level"
            );
        }
    }

    #[test]
    fn a_rounded_corner_keeps_its_ends_and_stays_inside_the_path() {
        let path = [[0.0, 0.0], [40.0, 0.0], [40.0, 30.0]];
        let curve = rounded(&path);
        assert_eq!(curve[0], path[0]);
        assert_eq!(curve[curve.len() - 1], path[2]);
        assert!(curve.len() > path.len());
        for p in &curve {
            assert!(
                p[0] >= -1e-4 && p[0] <= 40.0 + 1e-4,
                "{p:?} leaves the path"
            );
            assert!(
                p[1] >= -1e-4 && p[1] <= 30.0 + 1e-4,
                "{p:?} leaves the path"
            );
        }
    }

    #[test]
    fn a_straight_belt_is_not_subdivided() {
        let path = [[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]];
        assert_eq!(rounded(&path), path.to_vec());
    }

    #[test]
    fn machines_draw_back_to_front() {
        let at = |x: f32, y: f32| Placed {
            x,
            y,
            kind: Kind::Assembler,
        };
        let laid = Layout {
            machines: vec![at(200.0, 200.0), at(0.0, 0.0), at(200.0, 0.0)],
            belts: Vec::new(),
            bounds: [0.0, 0.0, 296.0, 286.0],
        };
        let scene = scene(&laid);
        let offsets: Vec<[f32; 2]> = scene.instances.iter().map(|inst| inst.offset).collect();
        assert_eq!(
            offsets,
            vec![
                project(0.0, 0.0, 0.0),
                project(200.0, 0.0, 0.0),
                project(200.0, 200.0, 0.0),
            ]
        );
    }

    #[test]
    fn the_scene_has_one_instance_per_machine() {
        let plant = stress(200);
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        assert_eq!(scene(&laid).instances.len(), laid.machines.len());
    }

    #[test]
    fn the_floor_covers_the_plant_with_a_margin() {
        let laid = layout(
            &[Kind::Crate, Kind::Crate],
            &[Edge {
                from: 0,
                to: 1,
                imports: 1,
            }],
        );
        let floor = floor_rect(laid.bounds);
        assert!(floor[0] <= laid.bounds[0] - FLOOR_MARGIN);
        assert!(floor[2] >= laid.bounds[2] + FLOOR_MARGIN);
        assert_eq!(floor[0] % GRID, 0.0);
    }
}
