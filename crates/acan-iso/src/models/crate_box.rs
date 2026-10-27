//! The crate: a plain box with two bands and an inset lid line. It stands for anything not yet recognised.
//!
//! The reference puts a variant letter on the lid. The contract has no text item, so the lid is left plain.

use crate::figure::{Figure, Ink};
use crate::pen::{Look, Pen, Stroke};
use crate::plan::Plan;
use crate::scene::Scene;

const HEIGHT: f32 = 26.0;

pub(crate) fn build() -> Figure {
    let mut scene = Scene::new();

    let mut floor = Pen::new();
    floor.halo(
        &Plan {
            x: -16.0,
            y: -16.0,
            w: 32.0,
            d: 32.0,
            r: 2.0,
        }
        .ring(8),
        0.0,
        HEIGHT,
    );
    scene.floor(floor);

    let side = Plan {
        x: -16.0,
        y: -16.0,
        w: 32.0,
        d: 32.0,
        r: 1.5,
    };
    let mut part = Pen::new();
    part.solid_with(
        &side.ring(2),
        0.0,
        HEIGHT,
        Look::new(Ink::Mid).bevel(0.6),
        |pen| {
            for z in [9.0, 18.0] {
                pen.seam(&side.ring(2), z, Stroke::new(Ink::Lo).near());
            }
        },
    );
    part.plan_outline(side.inset(3.0), HEIGHT, 2, Stroke::new(Ink::Faint));
    part.screws(side, 2.5, HEIGHT);
    scene.put([0.0, 0.0, 13.0], part, vec![]);

    scene.finish()
}
