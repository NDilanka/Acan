//! Stand-in models and belts, until acan-iso supplies the real models. A model is a beveled
//! plinth with either a two-slab body or a cylinder. Detail is painted on: a panel line on a
//! slab top or a seam round a cylinder at the Near level and screws at the Close level, so no small step in the mesh draws a
//! doubled line. Models follow the shared contract: plan units, z up, origin at the plan
//! centre on the ground.

use acan_render::model::{Aabb, Detail, Dot3, Ink, Line3, Mesh, Model, Surface};
use acan_render::{Instance, Ribbon, Scene};

use crate::bends;
use crate::kind::Kind;
use crate::layout::{Belt, Layout};

/// The chamfer on the plinth's top edge: how far it steps in, and how far it rises.
const BEVEL_RUN: f32 = 1.2;
const BEVEL_RISE: f32 = 1.6;
const CYLINDER_SEGMENTS: usize = 32;
/// The upper slab steps in this far from the lower one, which leaves a ledge.
const SLAB_STEP: f32 = 3.0;
/// The painted panel on a slab top sits this far inside the top's edge.
const PANEL_INSET: f32 = 4.0;
const SCREW_RADIUS: f32 = 1.3;
/// Width of a detail line in device pixels. The renderer draws nothing thinner than this.
const LINE_WIDTH: f32 = 0.75;
/// A belt stops this far short of a plinth, so its end line reads as a port on the plate edge.
const BELT_PORT_GAP: f32 = 1.5;

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

/// The solid parts of a kind that can hide a label behind them, in model space: the plinth,
/// and the body standing on it. They are tighter than the model's box, which would also hide
/// labels in the empty air above a plinth's corners.
pub fn occluders(kind: Kind) -> [Aabb; 2] {
    let (w, d) = kind.footprint();
    let s = shape(kind);
    let (hx, hy) = (w / 2.0, d / 2.0);
    let (bx, by) = match s.form {
        Form::Slabs { inset } => (hx - inset, hy - inset),
        Form::Cylinder { radius } => {
            let r = radius * w.min(d);
            (r, r)
        }
    };
    [
        Aabb {
            min: [-hx, -hy, 0.0],
            max: [hx, hy, s.plinth],
        },
        Aabb {
            min: [-bx, -by, s.plinth],
            max: [bx, by, s.plinth + s.body],
        },
    ]
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
    let mut lines = Vec::new();
    let mut dots = Vec::new();

    plinth(&mut base, hx, hy, s.plinth);
    let top = match s.form {
        Form::Slabs { inset } => {
            let top = two_slabs(&mut base, hx, hy, s.plinth, s.body, inset);
            let (ux, uy) = (hx - inset - SLAB_STEP, hy - inset - SLAB_STEP);
            let (px, py) = (ux - PANEL_INSET, uy - PANEL_INSET);
            lines.push(stroke(
                vec![
                    [-px, -py, top],
                    [px, -py, top],
                    [px, py, top],
                    [-px, py, top],
                ],
                true,
                Ink::Faint,
                Detail::Near,
            ));
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
            // A painted circle inside the top would read as a second rim, so a cylinder's
            // only top line is its crease.
            lines.push(stroke(circle(r, seam), true, Ink::Faint, Detail::Near));
            for k in 0..s.screws {
                let a = std::f32::consts::TAU * k as f32 / s.screws as f32;
                let (sn, cs) = a.sin_cos();
                dots.push(screw(0.6 * r * cs, 0.6 * r * sn, top));
            }
            top
        }
    };

    let meshes = [base.finish(), Mesh::default(), Mesh::default()];
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

/// A flat screw head on the top surface at `(x, y)`.
fn screw(x: f32, y: f32, z: f32) -> Dot3 {
    Dot3 {
        center: [x, y, z],
        normal: [0.0, 0.0, 1.0],
        radius: SCREW_RADIUS,
        tone: Ink::Lo,
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

/// The plant: a ribbon for each belt and an instance for each machine.
pub fn scene(layout: &Layout, models: &[Model]) -> Scene {
    let ribbons = belt_ribbons(&layout.belts);
    let mut bounds = Aabb::EMPTY;
    for ribbon in &ribbons {
        for &[x, y] in &ribbon.centre {
            bounds.include([x, y, 0.0]);
        }
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
        ribbons,
        instances,
        bounds,
    }
}

/// Each belt is a flat ribbon on the ground along its rounded path, stopped just short of both
/// plinths. A belt too short to draw is left out.
fn belt_ribbons(belts: &[Belt]) -> Vec<Ribbon> {
    let (paths, widths): (Vec<Vec<[f32; 2]>>, Vec<f32>) = belts
        .iter()
        .filter_map(|belt| {
            Some((
                trimmed(&belt.path, BELT_PORT_GAP)?,
                belt_width(belt.imports),
            ))
        })
        .unzip();
    bends::round_all(&paths)
        .into_iter()
        .zip(widths)
        .map(|(centre, width)| Ribbon { centre, width })
        .collect()
}

/// A belt's width in plan units, a little wider for more imports. The widest belt still leaves
/// a gap to its neighbour at the layout's belt step.
pub fn belt_width(imports: u32) -> f32 {
    4.0 + imports.min(8) as f32 * 0.3
}

/// The path with `gap` cut off each end, or `None` when nothing is left.
fn trimmed(path: &[[f32; 2]], gap: f32) -> Option<Vec<[f32; 2]>> {
    let [first, second] = [*path.first()?, *path.get(1)?];
    let n = path.len();
    let [before_last, last] = [path[n - 2], path[n - 1]];
    let (head, tail) = (distance(first, second), distance(before_last, last));
    if head <= gap || tail <= gap || (n == 2 && head <= 2.0 * gap) {
        return None;
    }
    let mut out = path.to_vec();
    out[0] = toward(first, second, gap / head);
    out[n - 1] = toward(last, before_last, gap / tail);
    Some(out)
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
    fn every_kind_has_a_valid_base_and_painted_detail_at_each_level() {
        let models = models();
        assert_eq!(models.len(), Kind::ALL.len());
        for (kind, model) in Kind::ALL.iter().zip(&models) {
            assert!(
                !model.meshes[0].indices.is_empty(),
                "{kind:?} has no base mesh"
            );
            for mesh in &model.meshes {
                check_mesh(mesh);
            }
            assert!(
                model.lines.iter().any(|l| l.detail == Detail::Near),
                "{kind:?} paints no near detail"
            );
            assert!(
                model.dots.iter().all(|d| d.detail == Detail::Close) && !model.dots.is_empty(),
                "{kind:?} has no close screws"
            );
        }
    }

    #[test]
    fn a_cylinder_top_has_one_rim_line() {
        let model = model(Kind::Tank);
        let top = model.bounds.max[2];
        let r = 0.36 * 86.0;
        let rims = acan_render::edges::feature_lines(&model.meshes[0], Detail::Always)
            .into_iter()
            .chain(model.lines.iter().cloned())
            .filter(|line| {
                line.points
                    .iter()
                    .all(|p| (p[2] - top).abs() < 2.0 && (p[0].hypot(p[1]) - r).abs() < 3.0)
            })
            .count();
        assert_eq!(
            rims, CYLINDER_SEGMENTS,
            "one crease segment per side of the rim, nothing else"
        );
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
        assert_eq!(scene.ribbons.len(), laid.belts.len());
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
        for ribbon in &scene.ribbons {
            for p in &ribbon.centre {
                for (axis, value) in p.iter().enumerate() {
                    assert!(*value >= scene.bounds.min[axis] - 1e-3);
                    assert!(*value <= scene.bounds.max[axis] + 1e-3);
                }
            }
        }
    }

    #[test]
    fn a_belt_stops_just_short_of_both_plates() {
        let belt = Belt {
            from: 0,
            to: 1,
            imports: 1,
            path: vec![[0.0, 0.0], [100.0, 0.0]],
        };
        let ribbons = belt_ribbons(std::slice::from_ref(&belt));
        let ribbon = ribbons.first().expect("a long belt draws");
        assert_eq!(
            ribbon.centre,
            vec![[BELT_PORT_GAP, 0.0], [100.0 - BELT_PORT_GAP, 0.0]]
        );
        assert_eq!(ribbon.width, belt_width(1));
        let stub = Belt {
            path: vec![[0.0, 0.0], [2.0, 0.0]],
            ..belt
        };
        assert!(
            belt_ribbons(&[stub]).is_empty(),
            "a belt shorter than its gaps draws nothing"
        );
    }

    #[test]
    fn belts_are_narrow_next_to_the_machines() {
        let (w, d) = Kind::Assembler.footprint();
        assert!(
            belt_width(8) * 10.0 < w.min(d),
            "the widest belt stays under a tenth of a plate"
        );
        assert!(belt_width(1) < belt_width(8));
    }
}
