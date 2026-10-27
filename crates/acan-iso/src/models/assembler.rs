//! The assembler: a service that runs as a stack of boards, with a gauge, a knob, two dials and a lamp.
//!
//! Parts sit at their resting heights, which is the reference's zero explode. Every part is in plan
//! coordinates centred on the origin, so the mounted parts carry their own absolute positions.

use crate::figure::{Detail, Figure, Ink};
use crate::parts::Ticks;
use crate::pen::{Look, Pen, Stroke};
use crate::plan::{Plan, circle_ring};
use crate::scene::Scene;

/// The top of the shell. The gauge, knob, dials and lamp all stand on it.
const SHELL: f32 = 40.0;
/// Where the gauge needle points, from 0 at the left stop to 1 at the right stop.
const NEEDLE: f32 = 0.38;
const GAUGE: [f32; 2] = [18.0, 14.0];
const KNOB: [f32; 2] = [-12.0, -10.0];
const DIALS: [[f32; 2]; 2] = [[-30.0, 22.0], [-30.0, 8.0]];
const LAMP: [f32; 2] = [32.0, -28.0];

pub(crate) fn build() -> Figure {
    let mut scene = Scene::new();

    let mut floor = Pen::new();
    floor.plinth(0.0, 0.0, 96.0, 86.0);
    scene.floor(floor);

    let body = Plan {
        x: -38.0,
        y: -34.0,
        w: 76.0,
        d: 68.0,
        r: 4.0,
    };
    let board = Look::new(Ink::Mid).bevel(0.8);

    let mut base = Pen::new();
    base.solid(&body.ring(4), 4.0, 12.0, board);
    scene.put([0.0, 0.0, 10.0], base, vec![]);

    let mut upper = Pen::new();
    upper.solid(&body.ring(4), 16.0, 14.0, board);
    scene.put([0.0, 0.0, 23.0], upper, vec![]);

    let mut shell = Pen::new();
    shell.solid(&body.ring(4), 30.0, 10.0, board);
    shell.plan_outline(body.inset(4.0), SHELL, 4, Stroke::new(Ink::Faint));
    shell.screws(body, 3.5, SHELL);
    shell.dot_grid([6.0, -30.0], (8, 2), 3.2, SHELL, 0.45, Ink::Lo);
    scene.put(
        [0.0, 0.0, 35.0],
        shell,
        vec![
            ([GAUGE[0], GAUGE[1], SHELL + 2.0], gauge()),
            ([KNOB[0], KNOB[1], SHELL + 10.0], knob()),
            ([DIALS[0][0], DIALS[0][1], SHELL + 1.0], dial(DIALS[0])),
            ([DIALS[1][0], DIALS[1][1], SHELL + 1.0], dial(DIALS[1])),
            ([LAMP[0], LAMP[1], SHELL + 1.0], lamp()),
        ],
    );

    scene.finish()
}

/// The dial gauge: a puck with a knurled band, a scale around its top, and a needle.
fn gauge() -> Pen {
    let mut pen = Pen::new();
    pen.solid(
        &circle_ring(GAUGE[0], GAUGE[1], 13.0, 56),
        SHELL,
        5.0,
        Look::new(Ink::Hi).crease(Some(Ink::Lo)).bevel(0.6),
    );
    pen.ring(GAUGE, 11.0, SHELL + 5.0, 56, Stroke::new(Ink::Lo));
    let ticks = Ticks {
        from: -2.4,
        span: 4.8,
        ..Ticks::full(GAUGE, 10.0, 40, 5, SHELL + 5.0, [1.2, 2.6])
    };
    pen.radial_ticks(
        ticks,
        Stroke::new(Ink::Lo).close(),
        Stroke::new(Ink::Mid).near(),
    );
    let angle = -2.4 + 4.8 * NEEDLE;
    let tip = [GAUGE[0] + angle.sin() * 9.0, GAUGE[1] - angle.cos() * 9.0];
    pen.on_top(
        &[GAUGE, tip],
        SHELL + 5.2,
        false,
        Stroke::new(Ink::Lit).width(1.1),
    );
    pen.dots(
        &[[GAUGE[0], GAUGE[1], SHELL + 5.3]],
        0.9,
        Ink::Hi,
        Detail::Always,
    );
    pen
}

/// The knob: a flange, a knurled stem with a pointer, and the cap of the box it sits on.
fn knob() -> Pen {
    let mut pen = Pen::new();
    pen.solid(
        &circle_ring(KNOB[0], KNOB[1], 10.0, 32),
        SHELL,
        4.0,
        Look::new(Ink::Mid).bevel(0.5),
    );
    pen.solid(
        &circle_ring(KNOB[0], KNOB[1], 5.0, 28),
        SHELL + 4.0,
        14.0,
        Look::new(Ink::Mid),
    );
    pen.knurl(KNOB, 5.0, SHELL + 6.0, SHELL + 16.0, 18, Ink::Lo);
    let cap = Plan {
        x: -24.0,
        y: -22.0,
        w: 24.0,
        d: 24.0,
        r: 1.5,
    };
    pen.solid(
        &cap.ring(3),
        SHELL + 18.0,
        6.0,
        Look::new(Ink::Hi).crease(Some(Ink::Lo)).bevel(0.6),
    );
    pen.screws(cap, 3.0, SHELL + 24.0);
    pen
}

/// A small dial: a disc with 12 ticks and a hand.
fn dial(center: [f32; 2]) -> Pen {
    let [ox, oy] = center;
    let mut pen = Pen::new();
    pen.solid(
        &circle_ring(ox, oy, 5.5, 28),
        SHELL,
        2.4,
        Look::new(Ink::Mid).bevel(0.4),
    );
    let ticks = Ticks::full(center, 4.8, 12, 3, SHELL + 2.4, [0.8, 1.4]);
    pen.radial_ticks(
        ticks,
        Stroke::new(Ink::Lo).close(),
        Stroke::new(Ink::Mid).near(),
    );
    pen.on_top(
        &[center, [ox + 3.6, oy - 1.8]],
        SHELL + 2.5,
        false,
        Stroke::new(Ink::Hi),
    );
    pen
}

fn lamp() -> Pen {
    let mut pen = Pen::new();
    pen.lamp(LAMP, SHELL);
    pen
}
