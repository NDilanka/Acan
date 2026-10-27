//! The splitter: a round hub with a crown and dial, and four chutes that fan out to the corners.

use std::f32::consts::PI;

use crate::mesh::{Base, Finish, LIFT, Part, Stroke};
use crate::model::{Ink, Model};
use crate::parts::{self, Dial};

/// The hub's radius and top. The crown sits on the top.
const HUB_RADIUS: f32 = 14.0;
const HUB_TOP: f32 = 26.0;
/// The crown's top, where its dial and lamp stand.
const CROWN_TOP: f32 = 29.0;
/// The chutes' top.
const CHUTE_TOP: f32 = 13.0;

pub(crate) fn build() -> Model {
    let mut part = Part::new();
    parts::plinth(&mut part, 86.0, 86.0);
    chute(&mut part, PI, 26.0, 6.0);
    for angle in [-0.75, 0.0, 0.75] {
        chute(&mut part, angle, 24.0, 5.0);
    }
    hub(&mut part);
    crown(&mut part);
    part.finish()
}

/// One chute: a bevelled prism from the hub's wall outward along `angle`, with a faint groove and a tip dot. It
/// starts on the wall rather than inside the hub, so it does not sink into the drum.
fn chute(part: &mut Part, angle: f32, length: f32, wide: f32) {
    let (ux, uy) = (angle.cos(), angle.sin());
    let (vx, vy) = (-uy, ux);
    let (r0, r1) = (HUB_RADIUS, HUB_RADIUS + length);
    let across = |along: f32, side: f32| [ux * along + vx * side, uy * along + vy * side];
    let ring = [
        across(r0, -wide),
        across(r1, -wide),
        across(r1, wide),
        across(r0, wide),
    ];
    part.slab(&Base::polygon(&ring), 3.0, CHUTE_TOP, 0.3, Finish::paper());
    let tip = [ux * (r1 - 3.0), uy * (r1 - 3.0)];
    part.line(
        &[
            [ux * (r0 + 2.0), uy * (r0 + 2.0), CHUTE_TOP + LIFT],
            [tip[0], tip[1], CHUTE_TOP + LIFT],
        ],
        false,
        Stroke::new(Ink::Faint),
    );
    part.dot(
        [tip[0], tip[1], CHUTE_TOP + LIFT],
        [0.0, 0.0, 1.0],
        0.6,
        Stroke::new(Ink::Hi),
    );
}

/// The hub: a bevelled drum with a seam ring around its side.
fn hub(part: &mut Part) {
    part.slab(
        &Base::Circle {
            center: [0.0, 0.0],
            radius: HUB_RADIUS,
            steps: 40,
        },
        3.0,
        HUB_TOP,
        0.6,
        Finish::paper(),
    );
    parts::band(part, [0.0, 0.0], HUB_RADIUS, 16.0, 40, Stroke::new(Ink::Lo));
}

/// The crown: a short drum with a dial of ticks, a pointer, a centre pin and a lamp at its edge.
fn crown(part: &mut Part) {
    part.slab(
        &Base::Circle {
            center: [0.0, 0.0],
            radius: 9.0,
            steps: 32,
        },
        HUB_TOP,
        CROWN_TOP,
        0.4,
        Finish::paper(),
    );
    parts::dial_ticks(
        part,
        Dial::full([0.0, 0.0], 8.0, CROWN_TOP, 24, 6, [0.8, 1.8]),
        Stroke::new(Ink::Lo).close(),
        Stroke::new(Ink::Mid).near(),
    );
    part.line(
        &[[0.0, 0.0, CROWN_TOP + LIFT], [7.0, -3.0, CROWN_TOP + LIFT]],
        false,
        Stroke::new(Ink::Hi).width(1.1),
    );
    part.dot(
        [0.0, 0.0, CROWN_TOP + LIFT],
        [0.0, 0.0, 1.0],
        0.8,
        Stroke::new(Ink::Hi),
    );
    // Inside the crown's radius of 9, so the lamp does not overhang the edge.
    parts::lamp(part, [-5.5, 3.0], CROWN_TOP);
}
