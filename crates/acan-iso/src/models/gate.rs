//! The gate: a tall auth post with a centre hub of three arms, a block with a ring, and a head with a lamp.
//!
//! Drawn from the reference explore's gate at zero explode.

use std::f32::consts::TAU;

use crate::figure::{Detail, Figure, Ink};
use crate::pen::{Look, Pen, Stroke};
use crate::plan::{Plan, circle_ring};
use crate::scene::Scene;

pub(crate) fn build() -> Figure {
    let mut scene = Scene::new();

    let mut floor = Pen::new();
    floor.plinth(0.0, 0.0, 44.0, 74.0);
    scene.floor(floor);

    for y in [-26.0, 26.0] {
        let mut post = Pen::new();
        post.solid(
            &circle_ring(0.0, y, 4.0, 28),
            4.0,
            30.0,
            Look::new(Ink::Mid),
        );
        scene.put([0.0, y, 19.0], post, vec![]);
    }

    let mut hub = Pen::new();
    hub.solid(
        &circle_ring(0.0, 0.0, 3.2, 28),
        4.0,
        14.0,
        Look::new(Ink::Hi),
    );
    for angle in [0.0, TAU / 3.0, 2.0 * TAU / 3.0] {
        hub.on_top(
            &[[0.0, 0.0], [angle.sin() * 14.0, -angle.cos() * 14.0]],
            18.0,
            false,
            Stroke::new(Ink::Hi),
        );
    }
    hub.dots(&[[0.0, 0.0, 18.5]], 0.8, Ink::Hi, Detail::Always);
    scene.put([0.0, 0.0, 11.0], hub, vec![]);

    let mut block = Pen::new();
    block.solid(
        &Plan {
            x: 8.0,
            y: 10.0,
            w: 10.0,
            d: 12.0,
            r: 1.5,
        }
        .ring(2),
        4.0,
        14.0,
        Look::new(Ink::Mid).bevel(0.4),
    );
    block.ring([13.0, 16.0], 2.4, 18.0, 14, Stroke::new(Ink::Hi));
    scene.put([13.0, 16.0, 11.0], block, vec![]);

    let mut head = Pen::new();
    head.solid(
        &Plan {
            x: -5.0,
            y: -31.0,
            w: 10.0,
            d: 62.0,
            r: 1.5,
        }
        .ring(3),
        34.0,
        5.0,
        Look::new(Ink::Mid).bevel(0.5),
    );
    head.plan_outline(
        Plan {
            x: -3.0,
            y: -28.0,
            w: 6.0,
            d: 56.0,
            r: 1.0,
        },
        39.0,
        2,
        Stroke::new(Ink::Faint),
    );
    head.lamp([0.0, -20.0], 39.0);
    scene.put([0.0, 0.0, 36.5], head, vec![]);

    scene.finish()
}
