//! The splitter: a round hub with a crown and dial, and four chutes that fan out to the corners.
//!
//! Drawn from the reference explore's splitter.

use std::f32::consts::PI;

use crate::figure::{Detail, Figure, Ink};
use crate::parts::Ticks;
use crate::pen::{Look, Pen, Stroke};
use crate::plan::circle_ring;
use crate::scene::Scene;

/// The height the crown is mounted at on the hub.
const CROWN: f32 = 28.0;

pub(crate) fn build() -> Figure {
    let mut scene = Scene::new();

    let mut floor = Pen::new();
    floor.plinth(0.0, 0.0, 86.0, 86.0);
    scene.floor(floor);

    chute(&mut scene, PI, 28.0, 6.0);
    for angle in [-0.75, 0.0, 0.75] {
        chute(&mut scene, angle, 26.0, 5.0);
    }

    let mut hub = Pen::new();
    hub.solid(
        &circle_ring(0.0, 0.0, 14.0, 40),
        4.0,
        22.0,
        Look::new(Ink::Mid).bevel(0.6),
    );
    hub.side_arc([0.0, 0.0], 14.0, 16.0, 32, Stroke::new(Ink::Lo));
    scene.put([0.0, 0.0, 15.0], hub, vec![([0.0, 0.0, CROWN], crown())]);

    scene.finish()
}

/// One chute: a bevelled prism from the hub outward along `angle`, with a faint crease and a tip dot.
fn chute(scene: &mut Scene, angle: f32, length: f32, wide: f32) {
    let (ux, uy) = (angle.cos(), angle.sin());
    let (vx, vy) = (-uy, ux);
    let (r0, r1) = (10.0, 10.0 + length);
    let across = |along: f32, side: f32| [ux * along + vx * side, uy * along + vy * side];
    let ring = [
        across(r0, -wide),
        across(r1, -wide),
        across(r1, wide),
        across(r0, wide),
    ];
    let tip = [ux * (r1 - 3.0), uy * (r1 - 3.0)];
    let middle = r0 + length / 2.0;

    let mut part = Pen::new();
    part.solid(&ring, 4.0, 10.0, Look::new(Ink::Mid).bevel(0.3));
    part.on_top(
        &[[ux * (r0 + 2.0), uy * (r0 + 2.0)], tip],
        14.0,
        false,
        Stroke::new(Ink::Faint),
    );
    part.dots(&[[tip[0], tip[1], 14.0]], 0.6, Ink::Hi, Detail::Always);
    scene.put([ux * middle, uy * middle, 9.0], part, vec![]);
}

/// The crown: a short drum with a dial of ticks, a pointer, and a lamp at its edge.
fn crown() -> Pen {
    let mut pen = Pen::new();
    pen.solid(
        &circle_ring(0.0, 0.0, 9.0, 32),
        26.0,
        3.0,
        Look::new(Ink::Hi).crease(Some(Ink::Lo)).bevel(0.4),
    );
    let ticks = Ticks::full([0.0, 0.0], 8.0, 24, 6, 29.0, [0.8, 1.8]);
    pen.radial_ticks(
        ticks,
        Stroke::new(Ink::Lo).close(),
        Stroke::new(Ink::Mid).near(),
    );
    pen.on_top(
        &[[0.0, 0.0], [7.0, -3.0]],
        29.2,
        false,
        Stroke::new(Ink::Lit),
    );
    pen.dots(&[[0.0, 0.0, 29.3]], 0.8, Ink::Hi, Detail::Always);
    pen.lamp([-6.0, 4.0], 29.0);
    pen
}
