//! The gate: two auth posts carrying a lintel, a three-arm turnstile hub between them, and a badge reader box.
//!
//! The turnstile arms are solid bars at hub height, each capped with a grip, so the hub reads as a solid rather
//! than a set of lines.

use std::f32::consts::TAU;

use crate::mesh::{Base, Finish, LIFT, Part, Stroke};
use crate::model::{Ink, Model};
use crate::parts;
use crate::plan::Plan;

/// The top of the posts, where the lintel rests.
const POST_TOP: f32 = 34.0;
/// The top of the lintel. The lamp stands on it and must stay under the kit's footprint top of 40.
const LINTEL_TOP: f32 = 38.0;
/// The bottom and top of the turnstile arms.
const ARM: [f32; 2] = [12.0, 14.2];

pub(crate) fn build() -> Model {
    let mut part = Part::new();
    parts::plinth(&mut part, 44.0, 74.0);
    for y in [-26.0, 26.0] {
        post(&mut part, y);
    }
    turnstile(&mut part);
    badge_reader(&mut part);
    lintel(&mut part);
    part.finish()
}

/// An auth post on a flange, with a cap under the lintel.
fn post(part: &mut Part, y: f32) {
    let centre = [0.0, y];
    part.slab(
        &Base::Circle {
            center: centre,
            radius: 5.2,
            steps: 28,
        },
        3.0,
        4.2,
        0.3,
        Finish::paper(),
    );
    part.slab(
        &Base::Circle {
            center: centre,
            radius: 4.0,
            steps: 28,
        },
        4.2,
        33.6,
        0.0,
        Finish::paper(),
    );
    part.slab(
        &Base::Circle {
            center: centre,
            radius: 4.5,
            steps: 28,
        },
        33.6,
        POST_TOP,
        0.1,
        Finish::paper(),
    );
}

/// The hub column, its cap, and three arms at hub height with a grip on each tip.
fn turnstile(part: &mut Part) {
    let origin = [0.0, 0.0];
    part.slab(
        &Base::Circle {
            center: origin,
            radius: 3.2,
            steps: 28,
        },
        3.0,
        18.0,
        0.0,
        Finish::paper(),
    );
    part.slab(
        &Base::Circle {
            center: origin,
            radius: 3.6,
            steps: 28,
        },
        18.0,
        18.8,
        0.3,
        Finish::paper(),
    );
    part.dot(
        [0.0, 0.0, 18.8 + LIFT],
        [0.0, 0.0, 1.0],
        0.8,
        Stroke::new(Ink::Hi),
    );
    for angle in [0.0, TAU / 3.0, 2.0 * TAU / 3.0] {
        let (sin, cos) = angle.sin_cos();
        let root = [sin * 2.2, -cos * 2.2];
        let tip = [sin * 14.0, -cos * 14.0];
        parts::bar(part, root, tip, 1.8, ARM[0], ARM[1], Finish::paper());
        // The grips are fine parts, so they belong to the `Near` mesh.
        part.slab(
            &Base::Circle {
                center: tip,
                radius: 1.3,
                steps: 16,
            },
            ARM[0],
            ARM[1],
            0.0,
            Finish::paper().near(),
        );
    }
}

/// The badge reader: a bevelled box with a ring on its top and a green LED in the ring.
fn badge_reader(part: &mut Part) {
    let block = Plan {
        x: 8.0,
        y: 10.0,
        w: 10.0,
        d: 12.0,
        r: 1.5,
    };
    part.slab(&Base::Plan(block, 2), 3.0, 18.0, 0.4, Finish::paper());
    parts::circle_line(part, [13.0, 16.0], 2.4, 18.0, 16, Stroke::new(Ink::Hi));
    part.dot(
        [13.0, 16.0, 18.0 + LIFT],
        [0.0, 0.0, 1.0],
        0.8,
        Stroke::new(Ink::Mid),
    );
}

/// The lintel: a bevelled beam across the posts, with a groove along its top and a lamp.
fn lintel(part: &mut Part) {
    let beam = Plan {
        x: -5.0,
        y: -31.0,
        w: 10.0,
        d: 62.0,
        r: 1.5,
    };
    part.slab(
        &Base::Plan(beam, 3),
        POST_TOP,
        LINTEL_TOP,
        0.5,
        Finish::paper(),
    );
    parts::plan_line(
        part,
        beam.inset(2.5),
        LINTEL_TOP,
        2,
        Stroke::new(Ink::Faint),
    );
    parts::lamp(part, [0.0, -20.0], LINTEL_TOP);
}
