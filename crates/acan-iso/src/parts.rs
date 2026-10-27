//! Parts that recur across machines: the plinth, outlines, screws, ticks, knurls, seams and lamps.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::figure::{Detail, Ink};
use crate::geom::{Point, facing_edges, runs_of};
use crate::pen::{Look, Pen, Stroke};
use crate::plan::{Plan, circle_ring};

/// Radius of a screw head, in plan units.
const SCREW: f32 = 0.5;
/// How far a knurl line stops short of its band, so the ridges do not touch the rings above and below.
const KNURL_INSET: f32 = 0.4;

/// Ticks around a dial: each tick runs from `radius - length` to `radius`, every `every`-th one is major.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Ticks {
    pub(crate) center: Point,
    pub(crate) radius: f32,
    pub(crate) count: usize,
    pub(crate) every: usize,
    pub(crate) z: f32,
    /// Minor and major tick lengths.
    pub(crate) lengths: [f32; 2],
    /// The angle of the first tick and the arc they cover, in radians. Zero and a full turn go all the way round.
    pub(crate) from: f32,
    pub(crate) span: f32,
}

impl Ticks {
    /// Ticks all the way round a dial.
    pub(crate) fn full(
        center: Point,
        radius: f32,
        count: usize,
        every: usize,
        z: f32,
        lengths: [f32; 2],
    ) -> Self {
        Self {
            center,
            radius,
            count,
            every,
            z,
            lengths,
            from: 0.0,
            span: TAU,
        }
    }
}

impl Pen {
    /// The base plate a machine stands on: a halo, a bevelled slab, a groove inside it, and screws at the corners.
    pub(crate) fn plinth(&mut self, cx: f32, cy: f32, w: f32, d: f32) {
        const HEIGHT: f32 = 3.0;
        let plan = Plan {
            x: cx - w / 2.0,
            y: cy - d / 2.0,
            w,
            d,
            r: 3.0,
        };
        self.halo(&plan.ring(8), 0.0, HEIGHT);
        self.solid(&plan.ring(4), 0.0, HEIGHT, Look::new(Ink::Lo).bevel(0.6));
        self.plan_outline(plan.inset(3.0), HEIGHT, 3, Stroke::new(Ink::Faint));
        self.screws(plan, 5.0, HEIGHT);
    }

    /// The outline of a rounded plan lying on the plane at height `z`.
    pub(crate) fn plan_outline(&mut self, plan: Plan, z: f32, steps: usize, stroke: Stroke) {
        self.on_top(&plan.ring(steps), z, true, stroke);
    }

    /// A circle lying on the plane at height `z`.
    pub(crate) fn ring(
        &mut self,
        center: Point,
        radius: f32,
        z: f32,
        steps: usize,
        stroke: Stroke,
    ) {
        self.on_top(
            &circle_ring(center[0], center[1], radius, steps),
            z,
            true,
            stroke,
        );
    }

    /// Screw heads at the four corners of `plan`, inset by `inset`.
    pub(crate) fn screws(&mut self, plan: Plan, inset: f32, z: f32) {
        let heads = plan.corners(inset).map(|[x, y]| [x, y, z]);
        self.dots(&heads, SCREW, Ink::Mid, Detail::Near);
    }

    /// A `columns` by `rows` grid of dots, `gap` apart, starting at `origin`.
    pub(crate) fn dot_grid(
        &mut self,
        origin: Point,
        (columns, rows): (usize, usize),
        gap: f32,
        z: f32,
        radius: f32,
        ink: Ink,
    ) {
        let mut points = Vec::with_capacity(columns * rows);
        for row in 0..rows {
            for column in 0..columns {
                points.push([
                    origin[0] + column as f32 * gap,
                    origin[1] + row as f32 * gap,
                    z,
                ]);
            }
        }
        self.dots(&points, radius, ink, Detail::Close);
    }

    /// Ridges down the side of a cylinder, on the faces that look toward the viewer.
    pub(crate) fn knurl(
        &mut self,
        center: Point,
        radius: f32,
        z0: f32,
        z1: f32,
        count: usize,
        ink: Ink,
    ) {
        let toward = self.cam.toward_viewer();
        for index in 0..count {
            let angle = index as f32 / count as f32 * TAU;
            let (nx, ny) = (angle.cos(), angle.sin());
            if nx * toward[0] + ny * toward[1] < 0.15 {
                continue;
            }
            let (x, y) = (center[0] + radius * nx, center[1] + radius * ny);
            self.line(
                &[[x, y, z0 + KNURL_INSET], [x, y, z1 - KNURL_INSET]],
                false,
                Stroke::new(ink).close(),
            );
        }
    }

    /// The runs of a plan's outline that face the viewer, lifted to `z`. Used as seams where a box's sides meet.
    pub(crate) fn seam(&mut self, ring: &[Point], z: f32, stroke: Stroke) {
        let (_, facing) = facing_edges(ring, self.cam.toward_viewer());
        for run in runs_of(&facing, true) {
            let points: Vec<[f32; 3]> = run.iter().map(|&i| [ring[i][0], ring[i][1], z]).collect();
            self.line(&points, false, stroke);
        }
    }

    /// An arc on the vertical side of a cylinder, through the middle of the side that faces the viewer.
    pub(crate) fn side_arc(
        &mut self,
        center: Point,
        radius: f32,
        z: f32,
        steps: usize,
        stroke: Stroke,
    ) {
        let toward = self.cam.toward_viewer();
        let facing = toward[1].atan2(toward[0]);
        let points: Vec<[f32; 3]> = (0..=steps)
            .map(|step| {
                let angle = facing - FRAC_PI_2 + step as f32 / steps as f32 * PI;
                [
                    center[0] + radius * angle.cos(),
                    center[1] + radius * angle.sin(),
                    z,
                ]
            })
            .collect();
        self.line(&points, false, stroke);
    }

    /// Radial ticks on a dial. Minor ticks and major ticks are drawn in separate strokes, minor first.
    pub(crate) fn radial_ticks(&mut self, ticks: Ticks, minor: Stroke, major: Stroke) {
        let closed = (ticks.span - TAU).abs() < 1e-6;
        let steps = if closed { ticks.count } else { ticks.count + 1 };
        let mut minors = Vec::new();
        let mut majors = Vec::new();
        for index in 0..steps {
            let angle = ticks.from + index as f32 / ticks.count as f32 * ticks.span;
            let big = index % ticks.every == 0;
            let inner = ticks.radius
                - if big {
                    ticks.lengths[1]
                } else {
                    ticks.lengths[0]
                };
            let (sx, sy) = (angle.sin(), -angle.cos());
            let [cx, cy] = ticks.center;
            let tick = [
                [cx + sx * inner, cy + sy * inner],
                [cx + sx * ticks.radius, cy + sy * ticks.radius],
            ];
            if big {
                majors.push(tick);
            } else {
                minors.push(tick);
            }
        }
        for tick in minors {
            self.on_top(&tick, ticks.z, false, minor);
        }
        for tick in majors {
            self.on_top(&tick, ticks.z, false, major);
        }
    }

    /// A small lamp: a short cylinder with no creases, lit at the top.
    pub(crate) fn lamp(&mut self, center: Point, z: f32) {
        self.solid(
            &circle_ring(center[0], center[1], 2.4, 16),
            z,
            1.8,
            Look::new(Ink::Hi).crease(None),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::Item;

    #[test]
    fn radial_ticks_count_and_split_minor_from_major() {
        let mut pen = Pen::new();
        let ticks = Ticks {
            from: -2.4,
            span: 4.8,
            ..Ticks::full([0.0, 0.0], 10.0, 40, 5, 5.0, [1.2, 2.6])
        };
        pen.radial_ticks(
            ticks,
            Stroke::new(Ink::Lo).close(),
            Stroke::new(Ink::Mid).near(),
        );
        // An open arc has count + 1 ticks, and every fifth is major.
        let majors = pen
            .items
            .iter()
            .filter(|i| matches!(i, Item::Line { tone: Ink::Mid, .. }))
            .count();
        let minors = pen
            .items
            .iter()
            .filter(|i| matches!(i, Item::Line { tone: Ink::Lo, .. }))
            .count();
        assert_eq!(majors, 9);
        assert_eq!(minors, 32);
    }

    #[test]
    fn full_dial_ticks_do_not_repeat_the_first() {
        let mut pen = Pen::new();
        pen.radial_ticks(
            Ticks::full([0.0, 0.0], 4.8, 12, 3, 2.4, [0.8, 1.4]),
            Stroke::new(Ink::Lo),
            Stroke::new(Ink::Mid),
        );
        let total = pen.items.len();
        assert_eq!(total, 12);
    }

    #[test]
    fn knurl_keeps_only_ridges_turned_toward_the_viewer() {
        // 18 ridges every 20 degrees; the viewer is at 45 degrees, so a ridge is kept when cos(angle - 45) >= 0.15.
        // That keeps 340, 0, 20, 40, 60, 80, 100 and 120 degrees, which is 8 ridges.
        let mut pen = Pen::new();
        pen.knurl([0.0, 0.0], 5.0, 0.0, 10.0, 18, Ink::Lo);
        assert_eq!(pen.items.len(), 8);
    }

    #[test]
    fn plinth_is_a_halo_then_a_slab_then_a_groove_and_screws() {
        let mut pen = Pen::new();
        pen.plinth(0.0, 0.0, 96.0, 86.0);
        assert!(matches!(
            pen.items[0],
            Item::Fill {
                tone: crate::figure::Fill::Halo,
                ..
            }
        ));
        assert!(matches!(pen.items.last(), Some(Item::Dots { .. })));
    }
}
