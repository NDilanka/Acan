//! Stand-in models and belt channels, until acan-iso supplies the real models. A model is a
//! beveled plinth with either a two-slab body or a cylinder, a lip or rim on top at the Near
//! level, and screws or a panel at the Close level. Models follow the shared contract: plan
//! units, z up, origin at the plan centre on the ground.

use acan_render::model::{Aabb, Detail, Dot3, Ink, Line3, Mesh, Model, Surface};
use acan_render::{Instance, Scene};

use crate::kind::Kind;
use crate::layout::{Belt, Layout};

/// The chamfer on the plinth's top edge: how far it steps in, and how far it rises.
const BEVEL_RUN: f32 = 1.2;
const BEVEL_RISE: f32 = 1.6;
const CYLINDER_SEGMENTS: usize = 32;
const RIM_WIDTH: f32 = 1.2;
const RIM_HEIGHT: f32 = 1.6;
/// The upper slab steps in this far from the lower one, which leaves a ledge.
const SLAB_STEP: f32 = 3.0;
const LIP_WIDTH: f32 = 2.0;
const LIP_HEIGHT: f32 = 1.2;
const PANEL_HEIGHT: f32 = 0.4;
const SCREW_RADIUS: f32 = 1.3;
const LINE_WIDTH: f32 = 0.6;
/// Depth of a belt channel below the grid, in plan units.
const BELT_DEPTH: f32 = 1.5;
/// Radius of a belt's rounded corners, in plan units.
const BELT_CORNER: f32 = 10.0;
/// Segments per rounded corner.
const CORNER_STEPS: usize = 4;

/// The body shape of a kind.
enum Form {
    /// Two stacked boxes, the upper one stepped in from the edge by `inset`.
    Slabs { inset: f32 },
    /// A cylinder with a rim, as a fraction of the footprint's short side.
    Cylinder { radius: f32 },
}

struct Shape {
    plinth: f32,
    body: f32,
    form: Form,
    screws: usize,
}

fn shape(kind: Kind) -> Shape {
    match kind {
        Kind::Intake => Shape {
            plinth: 6.0,
            body: 26.0,
            form: Form::Slabs { inset: 12.0 },
            screws: 2,
        },
        Kind::Gate => Shape {
            plinth: 6.0,
            body: 30.0,
            form: Form::Cylinder { radius: 0.30 },
            screws: 4,
        },
        Kind::Tank => Shape {
            plinth: 6.0,
            body: 44.0,
            form: Form::Cylinder { radius: 0.36 },
            screws: 4,
        },
        Kind::Splitter => Shape {
            plinth: 6.0,
            body: 22.0,
            form: Form::Slabs { inset: 12.0 },
            screws: 6,
        },
        Kind::Valve => Shape {
            plinth: 6.0,
            body: 28.0,
            form: Form::Cylinder { radius: 0.28 },
            screws: 2,
        },
        Kind::Assembler => Shape {
            plinth: 6.0,
            body: 34.0,
            form: Form::Slabs { inset: 12.0 },
            screws: 4,
        },
        Kind::Crate => Shape {
            plinth: 4.0,
            body: 18.0,
            form: Form::Slabs { inset: 6.0 },
            screws: 2,
        },
    }
}

/// One model per kind, in `Kind::ALL` order.
pub fn models() -> Vec<Model> {
    Kind::ALL.iter().map(|&kind| model(kind)).collect()
}

fn model(kind: Kind) -> Model {
    let (w, d) = kind.footprint();
    let s = shape(kind);
    let (hx, hy) = (w / 2.0, d / 2.0);
    let mut base = Builder::default();
    let mut near = Builder::default();
    let mut close = Builder::default();
    let mut lines = Vec::new();
    let mut dots = Vec::new();

    plinth(&mut base, hx, hy, s.plinth);
    let top = match s.form {
        Form::Slabs { inset } => {
            let top = two_slabs(&mut base, hx, hy, s.plinth, s.body, inset);
            let (ux, uy) = (hx - inset - SLAB_STEP, hy - inset - SLAB_STEP);
            let seam = top - 4.0;
            lines.push(stroke(
                vec![[-ux, uy, seam], [ux, uy, seam]],
                false,
                Ink::Lo,
                Detail::Always,
            ));
            let ticks = 5;
            for k in 0..ticks {
                let x = -ux + (k as f32 + 0.5) * (2.0 * ux / ticks as f32);
                lines.push(stroke(
                    vec![[x, uy, top - 9.0], [x, uy, top - 7.0]],
                    false,
                    Ink::Lo,
                    Detail::Near,
                ));
            }
            let outer = (ux - 1.5, uy - 1.5);
            let inner = (outer.0 - LIP_WIDTH, outer.1 - LIP_WIDTH);
            ring_lip(&mut near, outer, inner, top, LIP_HEIGHT);
            slab_panel(
                &mut close,
                (inner.0 - 1.5, inner.1 - 1.5),
                top,
                top + PANEL_HEIGHT,
            );
            let sites = [
                [ux - 4.0, uy - 4.0],
                [-ux + 4.0, uy - 4.0],
                [-ux + 4.0, -uy + 4.0],
                [ux - 4.0, -uy + 4.0],
                [0.0, uy - 4.0],
                [ux - 4.0, 0.0],
                [0.0, -uy + 4.0],
                [-ux + 4.0, 0.0],
            ];
            dots.extend(sites.iter().take(s.screws).map(|&[x, y]| screw(x, y, top)));
            top
        }
        Form::Cylinder { radius } => {
            let r = radius * w.min(d);
            let top = s.plinth + s.body;
            cylinder(&mut base, r, s.plinth, top);
            let seam = s.plinth + s.body / 2.0;
            lines.push(stroke(circle(r, seam), true, Ink::Lo, Detail::Always));
            lines.push(stroke(circle(r - 5.0, top), true, Ink::Faint, Detail::Near));
            rim(&mut near, r, top);
            cylinder_panel(&mut close, r - 3.0, top);
            for k in 0..s.screws {
                let a = std::f32::consts::TAU * k as f32 / s.screws as f32;
                let (sn, cs) = a.sin_cos();
                dots.push(screw(0.6 * r * cs, 0.6 * r * sn, top));
            }
            top
        }
    };

    let meshes = [base.finish(), near.finish(), close.finish()];
    let mut bounds = Aabb::EMPTY;
    for mesh in &meshes {
        for &p in &mesh.positions {
            bounds.include(p);
        }
    }
    for line in &lines {
        for &p in &line.points {
            bounds.include(p);
        }
    }
    for dot in &dots {
        bounds.include(dot.center);
    }
    debug_assert!(bounds.max[2] >= top);
    Model {
        meshes,
        lines,
        dots,
        bounds,
    }
}

#[derive(Default)]
struct Builder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    surfaces: Vec<Surface>,
    indices: Vec<u32>,
}

impl Builder {
    fn vertex(&mut self, p: [f32; 3], n: [f32; 3]) -> u32 {
        self.positions.push(p);
        self.normals.push(n);
        self.surfaces.push(Surface::Paper);
        (self.positions.len() - 1) as u32
    }

    /// A quad whose corners go around the face, each with its own normal.
    fn quad(&mut self, c: [[f32; 3]; 4], n: [[f32; 3]; 4]) {
        let i: [u32; 4] = std::array::from_fn(|k| self.vertex(c[k], n[k]));
        self.indices.extend([i[0], i[1], i[2], i[0], i[2], i[3]]);
    }

    /// A quad with one normal for the whole face.
    fn flat(&mut self, c: [[f32; 3]; 4], n: [f32; 3]) {
        self.quad(c, [n; 4]);
    }

    fn finish(self) -> Mesh {
        Mesh {
            positions: self.positions,
            normals: self.normals,
            surfaces: self.surfaces,
            indices: self.indices,
        }
    }
}

/// The four sides of a rectangle with half sizes `hx` and `hy`: two end points and the outward
/// normal of each side.
fn sides(hx: f32, hy: f32) -> [([f32; 2], [f32; 2], [f32; 2]); 4] {
    [
        ([hx, -hy], [hx, hy], [1.0, 0.0]),
        ([hx, hy], [-hx, hy], [0.0, 1.0]),
        ([-hx, hy], [-hx, -hy], [-1.0, 0.0]),
        ([-hx, -hy], [hx, -hy], [0.0, -1.0]),
    ]
}

/// The outward walls of a rectangle, from `z0` to `z1`.
fn slab(b: &mut Builder, half: (f32, f32), z0: f32, z1: f32) {
    for (a, c, n) in sides(half.0, half.1) {
        b.flat(
            [
                [a[0], a[1], z0],
                [c[0], c[1], z0],
                [c[0], c[1], z1],
                [a[0], a[1], z1],
            ],
            [n[0], n[1], 0.0],
        );
    }
}

/// A flat top: the rectangle at height `z`, facing up.
fn cap(b: &mut Builder, half: (f32, f32), z: f32) {
    let (hx, hy) = half;
    b.flat(
        [[-hx, -hy, z], [hx, -hy, z], [hx, hy, z], [-hx, hy, z]],
        [0.0, 0.0, 1.0],
    );
}

/// A flat ring at height `z` between an outer and an inner rectangle, facing up.
fn ring_cap(b: &mut Builder, outer: (f32, f32), inner: (f32, f32), z: f32) {
    let outer_sides = sides(outer.0, outer.1);
    let inner_sides = sides(inner.0, inner.1);
    for (&(oa, oc, _), &(ia, ic, _)) in outer_sides.iter().zip(&inner_sides) {
        b.flat(
            [
                [oa[0], oa[1], z],
                [oc[0], oc[1], z],
                [ic[0], ic[1], z],
                [ia[0], ia[1], z],
            ],
            [0.0, 0.0, 1.0],
        );
    }
}

/// A raised frame on a surface: outer walls, inner walls facing into the opening, and the top
/// between them. Its bottom lies on the surface below and is not drawn.
fn ring_lip(b: &mut Builder, outer: (f32, f32), inner: (f32, f32), z0: f32, height: f32) {
    slab(b, outer, z0, z0 + height);
    for (a, c, n) in sides(inner.0, inner.1) {
        b.flat(
            [
                [a[0], a[1], z0],
                [c[0], c[1], z0],
                [c[0], c[1], z0 + height],
                [a[0], a[1], z0 + height],
            ],
            [-n[0], -n[1], 0.0],
        );
    }
    ring_cap(b, outer, inner, z0 + height);
}

/// A low raised box on a surface, from `z0` up to a flat top at `z1`.
fn slab_panel(b: &mut Builder, half: (f32, f32), z0: f32, z1: f32) {
    slab(b, half, z0, z1);
    cap(b, half, z1);
}

/// A beveled plinth on the ground: vertical walls, a chamfer at the top, and a flat top.
fn plinth(b: &mut Builder, hx: f32, hy: f32, height: f32) {
    let wall_top = height - BEVEL_RISE;
    let outer = sides(hx, hy);
    let inner = sides(hx - BEVEL_RUN, hy - BEVEL_RUN);
    slab(b, (hx, hy), 0.0, wall_top);
    let length = BEVEL_RISE.hypot(BEVEL_RUN);
    for (&(a, c, n), &(ia, ic, _)) in outer.iter().zip(&inner) {
        let normal = [
            n[0] * BEVEL_RISE / length,
            n[1] * BEVEL_RISE / length,
            BEVEL_RUN / length,
        ];
        b.flat(
            [
                [a[0], a[1], wall_top],
                [c[0], c[1], wall_top],
                [ic[0], ic[1], height],
                [ia[0], ia[1], height],
            ],
            normal,
        );
    }
    cap(b, (hx - BEVEL_RUN, hy - BEVEL_RUN), height);
}

/// The lower slab from the plinth top to mid height, a ledge, then the upper slab to the top.
/// Returns the height of the top.
fn two_slabs(b: &mut Builder, hx: f32, hy: f32, base: f32, body: f32, inset: f32) -> f32 {
    let lower = (hx - inset, hy - inset);
    let upper = (lower.0 - SLAB_STEP, lower.1 - SLAB_STEP);
    let mid = base + body / 2.0;
    let top = base + body;
    slab(b, lower, base, mid);
    ring_cap(b, lower, upper, mid);
    slab(b, upper, mid, top);
    cap(b, upper, top);
    top
}

/// A cylinder from `z0` to `z1` with its flat top.
fn cylinder(b: &mut Builder, radius: f32, z0: f32, z1: f32) {
    radial_band(b, radius, z0, z1);
    radial_cap(b, radius, 0.0, z1);
}

/// The outside walls of a cylinder of `radius`, from `z0` to `z1`, with smooth normals.
fn radial_band(b: &mut Builder, radius: f32, z0: f32, z1: f32) {
    for k in 0..CYLINDER_SEGMENTS {
        let (s0, c0) = segment_angle(k);
        let (s1, c1) = segment_angle(k + 1);
        let (n0, n1) = ([c0, s0, 0.0], [c1, s1, 0.0]);
        b.quad(
            [
                [radius * c0, radius * s0, z0],
                [radius * c1, radius * s1, z0],
                [radius * c1, radius * s1, z1],
                [radius * c0, radius * s0, z1],
            ],
            [n0, n1, n1, n0],
        );
    }
}

/// A flat disc or annulus at height `z`, from `inner` out to `outer`, facing up.
fn radial_cap(b: &mut Builder, outer: f32, inner: f32, z: f32) {
    for k in 0..CYLINDER_SEGMENTS {
        let (s0, c0) = segment_angle(k);
        let (s1, c1) = segment_angle(k + 1);
        b.flat(
            [
                [outer * c0, outer * s0, z],
                [outer * c1, outer * s1, z],
                [inner * c1, inner * s1, z],
                [inner * c0, inner * s0, z],
            ],
            [0.0, 0.0, 1.0],
        );
    }
}

fn segment_angle(k: usize) -> (f32, f32) {
    let a = std::f32::consts::TAU * k as f32 / CYLINDER_SEGMENTS as f32;
    a.sin_cos()
}

/// A rim around the top of a cylinder of `radius`: a band outside the body, and its flat top.
fn rim(b: &mut Builder, radius: f32, top: f32) {
    let outer = radius + RIM_WIDTH;
    radial_band(b, outer, top, top + RIM_HEIGHT);
    radial_cap(b, outer, radius, top + RIM_HEIGHT);
}

/// A low disc on top of a cylinder, inside its rim.
fn cylinder_panel(b: &mut Builder, radius: f32, top: f32) {
    radial_band(b, radius, top, top + PANEL_HEIGHT);
    radial_cap(b, radius, 0.0, top + PANEL_HEIGHT);
}

/// A flat screw head on the top surface at `(x, y)`.
fn screw(x: f32, y: f32, z: f32) -> Dot3 {
    Dot3 {
        center: [x, y, z],
        normal: [0.0, 0.0, 1.0],
        radius: SCREW_RADIUS,
        tone: Ink::Hi,
        detail: Detail::Close,
    }
}

fn stroke(points: Vec<[f32; 3]>, closed: bool, tone: Ink, detail: Detail) -> Line3 {
    Line3 {
        points,
        closed,
        tone,
        width: LINE_WIDTH,
        detail,
    }
}

/// A circle of `radius` around the axis at height `z`, as a closed polyline.
fn circle(radius: f32, z: f32) -> Vec<[f32; 3]> {
    (0..CYLINDER_SEGMENTS)
        .map(|k| {
            let (s, c) = segment_angle(k);
            [radius * c, radius * s, z]
        })
        .collect()
}

/// The world drawn under the machines: a channel for each belt.
pub fn scene(layout: &Layout, models: &[Model]) -> Scene {
    let mut world = Builder::default();
    for belt in &layout.belts {
        channel(&mut world, belt);
    }
    let world = world.finish();
    let mut bounds = Aabb::EMPTY;
    for &p in &world.positions {
        bounds.include(p);
    }

    let mut instances = Vec::with_capacity(layout.machines.len());
    for placed in &layout.machines {
        let (w, d) = placed.kind.footprint();
        let offset = [placed.x + w / 2.0, placed.y + d / 2.0, 0.0];
        for corner in models[placed.kind.index()].bounds.corners() {
            bounds.include(std::array::from_fn(|axis| offset[axis] + corner[axis]));
        }
        instances.push(Instance {
            offset,
            kind: placed.kind.index() as u16,
            tint: 0,
        });
    }
    if bounds.min[0] > bounds.max[0] {
        bounds = Aabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
    }
    Scene {
        world,
        instances,
        bounds,
    }
}

/// A belt channel: a floor below the grid with a wall on each side.
fn channel(b: &mut Builder, belt: &Belt) {
    let centre = rounded(&belt.path);
    if centre.len() < 2 {
        return;
    }
    let normals = normals(&centre);
    let half = belt_width(belt.imports) / 2.0;
    let at = |p: [f32; 2], n: [f32; 2], side: f32, z: f32| {
        [p[0] + side * n[0] * half, p[1] + side * n[1] * half, z]
    };
    let floor = -BELT_DEPTH;
    for i in 0..centre.len() - 1 {
        let (c0, c1) = (centre[i], centre[i + 1]);
        let (n0, n1) = (normals[i], normals[i + 1]);
        b.flat(
            [
                at(c0, n0, 1.0, floor),
                at(c0, n0, -1.0, floor),
                at(c1, n1, -1.0, floor),
                at(c1, n1, 1.0, floor),
            ],
            [0.0, 0.0, 1.0],
        );
        for side in [1.0_f32, -1.0] {
            let inward = |n: [f32; 2]| [-side * n[0], -side * n[1], 0.0];
            b.quad(
                [
                    at(c0, n0, side, floor),
                    at(c1, n1, side, floor),
                    at(c1, n1, side, 0.0),
                    at(c0, n0, side, 0.0),
                ],
                [inward(n0), inward(n1), inward(n1), inward(n0)],
            );
        }
    }
}

fn belt_width(imports: u32) -> f32 {
    8.0 + imports.min(8) as f32 * 0.5
}

/// Replaces each right-angle corner with a short quadratic curve. The curve starts and ends
/// on the two segments, at most halfway along each, so neighbouring corners never overlap.
fn rounded(path: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if path.len() < 3 {
        return path.to_vec();
    }
    let mut out = vec![path[0]];
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
    out.push(path[path.len() - 1]);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::layout;
    use crate::plant::{from_scan_json, stress};

    const FIXTURE: &str = include_str!("../www/fixtures/demo-plant.json");

    fn check_mesh(mesh: &Mesh) {
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        assert_eq!(mesh.positions.len(), mesh.surfaces.len());
        assert_eq!(mesh.indices.len() % 3, 0);
        let count = mesh.positions.len() as u32;
        assert!(mesh.indices.iter().all(|&i| i < count));
        for n in &mesh.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-3, "normal {n:?} is not unit length");
        }
    }

    #[test]
    fn every_kind_has_all_three_levels_and_a_valid_base() {
        let models = models();
        assert_eq!(models.len(), Kind::ALL.len());
        for (kind, model) in Kind::ALL.iter().zip(&models) {
            assert!(
                !model.meshes[0].indices.is_empty(),
                "{kind:?} has no base mesh"
            );
            assert!(
                !model.meshes[1].indices.is_empty(),
                "{kind:?} has no near mesh"
            );
            assert!(
                !model.meshes[2].indices.is_empty(),
                "{kind:?} has no close mesh"
            );
            for mesh in &model.meshes {
                check_mesh(mesh);
            }
        }
    }

    #[test]
    fn models_stand_on_the_ground_and_fill_their_footprint() {
        for (kind, model) in Kind::ALL.iter().zip(models()) {
            let (w, d) = kind.footprint();
            assert!(
                model.bounds.min[2].abs() < 1e-4,
                "{kind:?} floats above z = 0"
            );
            assert!(model.bounds.max[2] > 6.0, "{kind:?} has no body");
            assert!((model.bounds.min[0] + w / 2.0).abs() < 1e-3);
            assert!((model.bounds.max[1] - d / 2.0).abs() < 1e-3);
        }
    }

    #[test]
    fn the_scene_places_each_machine_at_its_footprint_centre() {
        let plant = from_scan_json(FIXTURE).expect("fixture parses");
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let scene = scene(&laid, &models());
        assert_eq!(scene.instances.len(), plant.machines.len());
        for (placed, inst) in laid.machines.iter().zip(&scene.instances) {
            let (w, d) = placed.kind.footprint();
            assert_eq!(inst.offset, [placed.x + w / 2.0, placed.y + d / 2.0, 0.0]);
            assert_eq!(usize::from(inst.kind), placed.kind.index());
        }
        check_mesh(&scene.world);
    }

    #[test]
    fn the_scene_bounds_hold_every_machine_and_belt() {
        let plant = stress(200);
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let scene = scene(&laid, &models());
        for placed in &laid.machines {
            let f = placed.footprint();
            assert!(f[0] >= scene.bounds.min[0] && f[2] <= scene.bounds.max[0]);
            assert!(f[1] >= scene.bounds.min[1] && f[3] <= scene.bounds.max[1]);
        }
        for p in &scene.world.positions {
            for (axis, value) in p.iter().enumerate() {
                assert!(*value >= scene.bounds.min[axis] - 1e-3);
                assert!(*value <= scene.bounds.max[axis] + 1e-3);
            }
        }
    }

    #[test]
    fn a_belt_channel_dips_below_the_grid_and_stays_in_its_width() {
        let belt = Belt {
            from: 0,
            to: 1,
            imports: 1,
            path: vec![[0.0, 0.0], [100.0, 0.0]],
        };
        let mut world = Builder::default();
        channel(&mut world, &belt);
        let mesh = world.finish();
        let half = belt_width(1) / 2.0;
        for p in &mesh.positions {
            assert!(p[2] <= 1e-6 && p[2] >= -BELT_DEPTH - 1e-6);
            assert!(p[1].abs() <= half + 1e-4);
        }
    }

    #[test]
    fn rounding_keeps_the_endpoints_of_a_belt() {
        let path = [[0.0, 0.0], [50.0, 0.0], [50.0, 40.0]];
        let curve = rounded(&path);
        assert_eq!(curve.first(), Some(&[0.0, 0.0]));
        assert_eq!(curve.last(), Some(&[50.0, 40.0]));
    }
}
