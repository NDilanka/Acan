//! The assembler: a service that runs as a stack of boards, with a gauge, a knob, two dials and a lamp.
//!
//! Parts sit at their resting heights, the reference's zero explode, in plan coordinates centred on the origin.

use crate::mesh::{Base, Finish, LIFT, Part, Station, Stroke};
use crate::model::{Ink, Model};
use crate::parts::{self, Dial};
use crate::plan::{Plan, Point};

/// The top of the shell. The gauge, knob, dials and lamp all stand on it.
const SHELL: f32 = 40.0;
/// Where the gauge needle points, from 0 at the left stop to 1 at the right stop.
const NEEDLE: f32 = 0.38;
const GAUGE: Point = [18.0, 14.0];
const KNOB: Point = [-12.0, -10.0];
const DIALS: [Point; 2] = [[-30.0, 22.0], [-30.0, 8.0]];
const LAMP: Point = [32.0, -28.0];

pub(crate) fn build() -> Model {
    let mut part = Part::new();
    parts::plinth(&mut part, 96.0, 86.0);

    let body = Plan {
        x: -38.0,
        y: -34.0,
        w: 76.0,
        d: 68.0,
        r: 4.0,
    };
    // The three boards are one solid with a V-groove at each seam. Separate boards would share caps, which
    // makes the seams non-manifold.
    let boards = [
        Station::at(-0.8, 3.0),
        Station::at(0.0, 3.8),
        Station::at(0.0, 15.2),
        Station::at(-0.8, 16.0),
        Station::at(0.0, 16.8),
        Station::at(0.0, 29.2),
        Station::at(-0.8, 30.0),
        Station::at(0.0, 30.8),
        Station::at(0.0, 39.2),
        Station::at(-0.8, SHELL),
    ];
    part.sweep(&Base::Plan(body, 4), &boards, Finish::paper());
    parts::plan_line(
        &mut part,
        body.inset(4.0),
        SHELL,
        3,
        Stroke::new(Ink::Faint),
    );
    parts::screws(&mut part, body, 3.5, SHELL);
    for row in 0..2 {
        for column in 0..8 {
            part.dot(
                [
                    6.0 + column as f32 * 3.2,
                    -30.0 + row as f32 * 3.2,
                    SHELL + LIFT,
                ],
                [0.0, 0.0, 1.0],
                0.45,
                Stroke::new(Ink::Lo).close(),
            );
        }
    }

    gauge(&mut part);
    knob(&mut part);
    for centre in DIALS {
        dial(&mut part, centre);
    }
    parts::lamp(&mut part, LAMP, SHELL);
    part.finish()
}

/// The dial gauge: a puck with a seam ring round its top, a scale, and a needle.
fn gauge(part: &mut Part) {
    let top = SHELL + 5.0;
    part.slab(
        &Base::Circle {
            center: GAUGE,
            radius: 13.0,
            steps: 56,
        },
        SHELL,
        top,
        0.6,
        Finish::paper(),
    );
    parts::circle_line(part, GAUGE, 11.0, top, 56, Stroke::new(Ink::Lo));
    let scale = Dial {
        from: -2.4,
        span: 4.8,
        ..Dial::full(GAUGE, 10.0, top, 40, 5, [1.2, 2.6])
    };
    parts::dial_ticks(
        part,
        scale,
        Stroke::new(Ink::Lo).close(),
        Stroke::new(Ink::Mid).near(),
    );
    let angle = -2.4 + 4.8 * NEEDLE;
    let (sin, cos) = angle.sin_cos();
    let tip = [GAUGE[0] + sin * 9.0, GAUGE[1] - cos * 9.0];
    part.line(
        &[
            [GAUGE[0], GAUGE[1], top + LIFT],
            [tip[0], tip[1], top + LIFT],
        ],
        false,
        Stroke::new(Ink::Hi).width(1.1),
    );
    part.dot(
        [GAUGE[0], GAUGE[1], top + LIFT],
        [0.0, 0.0, 1.0],
        0.9,
        Stroke::new(Ink::Hi),
    );
}

/// The knob: a flange, a knurled stem, and the cap of the box it sits in.
fn knob(part: &mut Part) {
    part.slab(
        &Base::Circle {
            center: KNOB,
            radius: 10.0,
            steps: 32,
        },
        SHELL,
        SHELL + 4.0,
        0.5,
        Finish::paper(),
    );
    part.slab(
        &Base::Circle {
            center: KNOB,
            radius: 5.0,
            steps: 28,
        },
        SHELL + 4.0,
        SHELL + 18.0,
        0.0,
        Finish::paper(),
    );
    parts::knurl(
        part,
        KNOB,
        5.0,
        SHELL + 6.0,
        SHELL + 16.0,
        18,
        Stroke::new(Ink::Lo).close(),
    );
    let cap = Plan {
        x: -24.0,
        y: -22.0,
        w: 24.0,
        d: 24.0,
        r: 1.5,
    };
    part.slab(
        &Base::Plan(cap, 3),
        SHELL + 18.0,
        SHELL + 24.0,
        0.6,
        Finish::paper(),
    );
    parts::screws(part, cap, 3.0, SHELL + 24.0);
}

/// A small dial: a disc with 12 ticks and a hand.
fn dial(part: &mut Part, centre: Point) {
    let top = SHELL + 2.4;
    part.slab(
        &Base::Circle {
            center: centre,
            radius: 5.5,
            steps: 28,
        },
        SHELL,
        top,
        0.4,
        Finish::paper(),
    );
    parts::dial_ticks(
        part,
        Dial::full(centre, 4.8, top, 12, 3, [0.8, 1.4]),
        Stroke::new(Ink::Lo).close(),
        Stroke::new(Ink::Mid).near(),
    );
    let tip = [centre[0] + 3.6, centre[1] - 1.8];
    part.line(
        &[
            [centre[0], centre[1], top + LIFT],
            [tip[0], tip[1], top + LIFT],
        ],
        false,
        Stroke::new(Ink::Hi),
    );
}
