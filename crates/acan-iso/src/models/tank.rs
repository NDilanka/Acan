//! The tank: a relational database, drawn as a squat cylinder stack with banded seams, a gauge on its lid, and pipe
//! stubs on the side.
//!
//! The body is one lathe: a foot, three bands separated by beads, and a bevelled rim. The stubs and the lamp sit
//! outside it, and the plan is centred on the origin, so the body is offset to the left of centre.

use crate::mesh::{Base, Finish, LIFT, Part, Station, Stroke};
use crate::model::{Ink, Model};
use crate::parts::{self, Dial};
use crate::plan::Point;

/// The axis of the body. It is offset so that the body and the stubs together are centred on the origin.
const CENTRE: Point = [-9.5, 0.0];
const RADIUS: f32 = 21.0;
const STUB_RADIUS: f32 = 8.0;
/// The stubs' centres and their heights.
const STUBS: [(Point, f32); 2] = [([22.5, -14.0], 22.0), ([22.5, 14.0], 16.0)];
/// The pipes run along the stubs' centre lines at the height they share. Each starts at the body's wall (where
/// the wall is at y = ±14) and ends at the stub's side, so neither end sinks into the solid it meets.
const PIPE_Y: [f32; 2] = [-14.0, 14.0];
const PIPE_FROM_X: f32 = 6.2;
const PIPE_Z: f32 = 8.0;
const TOP: f32 = 50.0;
const LAMP: Point = [-26.0, -21.0];

pub(crate) fn build() -> Model {
    let mut part = Part::new();
    parts::plinth(&mut part, 78.0, 62.0);
    body(&mut part);
    gauge(&mut part);
    for (centre, height) in STUBS {
        stub(&mut part, centre, height);
    }
    for y in PIPE_Y {
        part.tube(
            [PIPE_FROM_X, y, PIPE_Z],
            [STUBS[0].0[0] - STUB_RADIUS, y, PIPE_Z],
            2.4,
            16,
            Finish::paper(),
        );
    }
    parts::lamp(&mut part, LAMP, 3.0);
    part.finish()
}

/// The body: a foot, three bands with beads between them, and a bevelled lid edge.
fn body(part: &mut Part) {
    let stations = [
        Station::at(-1.5, 3.0),
        Station::at(-1.5, 4.6),
        Station::at(0.0, 4.6),
        Station::at(0.0, 17.0),
        Station::at(0.9, 17.0),
        Station::at(0.9, 18.4),
        Station::at(0.0, 18.4),
        Station::at(0.0, 33.0),
        Station::at(0.9, 33.0),
        Station::at(0.9, 34.4),
        Station::at(0.0, 34.4),
        Station::at(0.0, 49.4),
        Station::at(-0.6, TOP),
    ];
    part.sweep(
        &Base::Circle {
            center: CENTRE,
            radius: RADIUS,
            steps: 36,
        },
        &stations,
        Finish::paper(),
    );
    for z in [11.0, 26.0, 42.0] {
        parts::band(part, CENTRE, RADIUS, z, 36, Stroke::new(Ink::Lo));
    }
}

/// The gauge on the lid: a dial disc with a ring, a scale and a needle.
fn gauge(part: &mut Part) {
    let top = TOP + 3.0;
    part.slab(
        &Base::Circle {
            center: CENTRE,
            radius: 15.0,
            steps: 48,
        },
        TOP,
        top,
        0.4,
        Finish::paper(),
    );
    parts::circle_line(part, CENTRE, 18.0, TOP, 48, Stroke::new(Ink::Faint));
    parts::dial_ticks(
        part,
        Dial::full(CENTRE, 13.0, top, 24, 6, [1.0, 2.2]),
        Stroke::new(Ink::Lo).close(),
        Stroke::new(Ink::Mid).near(),
    );
    let tip = [CENTRE[0] + 7.0, CENTRE[1] - 9.0];
    part.line(
        &[
            [CENTRE[0], CENTRE[1], top + LIFT],
            [tip[0], tip[1], top + LIFT],
        ],
        false,
        Stroke::new(Ink::Hi).width(1.1),
    );
}

/// A pipe stub: a short drum with a band round its middle and a ring on its cap.
fn stub(part: &mut Part, centre: Point, height: f32) {
    let base = 3.0;
    let top = base + height;
    part.slab(
        &Base::Circle {
            center: centre,
            radius: STUB_RADIUS,
            steps: 32,
        },
        base,
        top,
        0.0,
        Finish::paper(),
    );
    parts::band(
        part,
        centre,
        STUB_RADIUS,
        base + height / 2.0,
        32,
        Stroke::new(Ink::Lo),
    );
    parts::circle_line(part, centre, 5.0, top, 24, Stroke::new(Ink::Faint));
}
