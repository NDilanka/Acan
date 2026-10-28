//! Parts that recur across machines: the plinth, screws, lamps, dials, knurls, and the lines and bars they are made of.

use std::f32::consts::TAU;

use crate::mesh::{Base, Finish, LIFT, Part, Stroke};
use crate::model::Ink;
use crate::plan::{Plan, Point, circle_ring};

/// Radius of a screw head, in plan units.
const SCREW: f32 = 0.5;
/// How far a knurl ridge stops short of the rings above and below, so the ridges do not touch them.
const KNURL_INSET: f32 = 0.4;
/// The plinth's height.
pub(crate) const PLINTH: f32 = 3.0;

/// Ticks around a dial lying on the plane at `z`. Each tick runs from `radius - length` to `radius`, and every
/// `every`-th tick is major.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Dial {
    pub(crate) center: Point,
    pub(crate) radius: f32,
    pub(crate) z: f32,
    pub(crate) count: usize,
    pub(crate) every: usize,
    /// Minor and major tick lengths.
    pub(crate) lengths: [f32; 2],
    /// The angle of the first tick and the arc the ticks cover, in radians. A full turn goes all the way round.
    pub(crate) from: f32,
    pub(crate) span: f32,
}

impl Dial {
    /// Ticks all the way round a dial.
    pub(crate) fn full(
        center: Point,
        radius: f32,
        z: f32,
        count: usize,
        every: usize,
        lengths: [f32; 2],
    ) -> Self {
        Self {
            center,
            radius,
            z,
            count,
            every,
            lengths,
            from: 0.0,
            span: TAU,
        }
    }
}

/// A base plate `w` by `d` on the ground, bevelled, with a groove inset along its top and a screw at each corner.
pub(crate) fn plinth(part: &mut Part, w: f32, d: f32) {
    let plan = Plan {
        x: -w / 2.0,
        y: -d / 2.0,
        w,
        d,
        r: 3.0,
    };
    part.slab(&Base::Plan(plan, 4), 0.0, PLINTH, 0.6, Finish::paper());
    plan_line(part, plan.inset(3.0), PLINTH, 3, Stroke::new(Ink::Faint));
    screws(part, plan, 5.0, PLINTH);
}

/// Screw heads at the four corners of `plan`, inset by `inset`, on the plane at `z`.
pub(crate) fn screws(part: &mut Part, plan: Plan, inset: f32, z: f32) {
    for [x, y] in plan.corners(inset) {
        part.dot(
            [x, y, z + LIFT],
            [0.0, 0.0, 1.0],
            SCREW,
            Stroke::new(Ink::Mid).near(),
        );
    }
}

/// A lamp: a short cylinder with a lens on its top.
pub(crate) fn lamp(part: &mut Part, center: Point, z: f32) {
    let height = 1.8;
    part.slab(
        &Base::Circle {
            center,
            radius: 2.4,
            steps: 16,
        },
        z,
        z + height,
        0.0,
        Finish::paper(),
    );
    part.dot(
        [center[0], center[1], z + height + LIFT],
        [0.0, 0.0, 1.0],
        1.4,
        Stroke::new(Ink::Mid),
    );
}

/// Ticks on a dial. Minor ticks use the first stroke and major ticks the second.
pub(crate) fn dial_ticks(part: &mut Part, dial: Dial, minor: Stroke, major: Stroke) {
    let closed = (dial.span - TAU).abs() < 1e-6;
    let steps = if closed { dial.count } else { dial.count + 1 };
    let [cx, cy] = dial.center;
    let z = dial.z + LIFT;
    for index in 0..steps {
        let angle = dial.from + index as f32 / dial.count as f32 * dial.span;
        let major_tick = index % dial.every == 0;
        let length = if major_tick {
            dial.lengths[1]
        } else {
            dial.lengths[0]
        };
        let inner = dial.radius - length;
        let (sin, cos) = angle.sin_cos();
        let points = [
            [cx + sin * inner, cy - cos * inner, z],
            [cx + sin * dial.radius, cy - cos * dial.radius, z],
        ];
        let stroke = if major_tick { major } else { minor };
        part.line(&points, false, stroke);
    }
}

/// A closed plan outline lying on the plane at `z`.
pub(crate) fn plan_line(part: &mut Part, plan: Plan, z: f32, steps: usize, stroke: Stroke) {
    let points: Vec<[f32; 3]> = plan
        .ring(steps)
        .into_iter()
        .map(|[x, y]| [x, y, z + LIFT])
        .collect();
    part.line(&points, true, stroke);
}

/// A closed plan outline running around the side of a box at height `z`, just outside its wall.
pub(crate) fn plan_band(part: &mut Part, plan: Plan, z: f32, steps: usize, stroke: Stroke) {
    let points: Vec<[f32; 3]> = Base::Plan(plan, steps)
        .ring(LIFT)
        .into_iter()
        .map(|[x, y]| [x, y, z])
        .collect();
    part.line(&points, true, stroke);
}

/// A closed circle lying on the plane at `z`.
pub(crate) fn circle_line(
    part: &mut Part,
    center: Point,
    radius: f32,
    z: f32,
    steps: usize,
    stroke: Stroke,
) {
    let points: Vec<[f32; 3]> = circle_ring(center[0], center[1], radius, steps)
        .into_iter()
        .map(|[x, y]| [x, y, z + LIFT])
        .collect();
    part.line(&points, true, stroke);
}

/// A closed ring around the side of a cylinder of `radius`, at height `z`.
pub(crate) fn band(
    part: &mut Part,
    center: Point,
    radius: f32,
    z: f32,
    steps: usize,
    stroke: Stroke,
) {
    let points: Vec<[f32; 3]> = circle_ring(center[0], center[1], radius + LIFT, steps)
        .into_iter()
        .map(|[x, y]| [x, y, z])
        .collect();
    part.line(&points, true, stroke);
}

/// Ridges running up a cylinder's side, all the way round.
pub(crate) fn knurl(
    part: &mut Part,
    center: Point,
    radius: f32,
    z0: f32,
    z1: f32,
    count: usize,
    stroke: Stroke,
) {
    for index in 0..count {
        let angle = index as f32 / count as f32 * TAU;
        let (sin, cos) = angle.sin_cos();
        let reach = radius + LIFT;
        let x = center[0] + reach * cos;
        let y = center[1] + reach * sin;
        part.line(
            &[[x, y, z0 + KNURL_INSET], [x, y, z1 - KNURL_INSET]],
            false,
            stroke,
        );
    }
}

/// A square-ended bar from `a` to `b` in plan, `width` across, from `z0` to `z1`.
pub(crate) fn bar(
    part: &mut Part,
    a: Point,
    b: Point,
    width: f32,
    z0: f32,
    z1: f32,
    finish: Finish,
) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let length = dx.hypot(dy);
    let (side_x, side_y) = (-dy / length * width / 2.0, dx / length * width / 2.0);
    let corners = [
        [a[0] + side_x, a[1] + side_y],
        [b[0] + side_x, b[1] + side_y],
        [b[0] - side_x, b[1] - side_y],
        [a[0] - side_x, a[1] - side_y],
    ];
    part.slab(&Base::polygon(&corners), z0, z1, 0.3, finish);
}
