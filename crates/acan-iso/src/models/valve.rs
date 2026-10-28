//! The valve: a small belt fitting with a body between two end caps, and a handwheel on a stem.

use std::f32::consts::FRAC_PI_3;

use crate::mesh::{Base, Finish, LIFT, Part, Stroke};
use crate::model::{Ink, Model};
use crate::parts;
use crate::plan::Plan;

/// The stem runs from the body to the wheel. The wheel and its spokes stand on it.
const STEM_TOP: f32 = 13.0;
const WHEEL: [f32; 2] = [13.0, 14.6];
const SPOKES_TOP: f32 = 15.6;

pub(crate) fn build() -> Model {
    let mut part = Part::new();
    part.slab(
        &Base::Plan(
            Plan {
                x: -8.0,
                y: -8.0,
                w: 16.0,
                d: 16.0,
                r: 1.5,
            },
            2,
        ),
        0.0,
        1.5,
        0.4,
        Finish::paper(),
    );
    part.slab(
        &Base::Plan(
            Plan {
                x: -5.0,
                y: -5.0,
                w: 10.0,
                d: 10.0,
                r: 1.5,
            },
            2,
        ),
        1.5,
        8.0,
        0.5,
        Finish::paper(),
    );
    for side in [-1.0_f32, 1.0] {
        end_cap(&mut part, side);
    }
    part.slab(
        &Base::Circle {
            center: [0.0, 0.0],
            radius: 1.6,
            steps: 12,
        },
        8.0,
        STEM_TOP,
        0.0,
        Finish::paper(),
    );
    part.slab(
        &Base::Circle {
            center: [0.0, 0.0],
            radius: 6.5,
            steps: 28,
        },
        WHEEL[0],
        WHEEL[1],
        0.4,
        Finish::paper(),
    );
    for index in 0..3 {
        let angle = index as f32 * FRAC_PI_3;
        let (sin, cos) = angle.sin_cos();
        parts::bar(
            &mut part,
            [-6.0 * sin, 6.0 * cos],
            [6.0 * sin, -6.0 * cos],
            1.0,
            WHEEL[1],
            SPOKES_TOP,
            Finish::paper(),
        );
    }
    part.dot(
        [0.0, 0.0, SPOKES_TOP + LIFT],
        [0.0, 0.0, 1.0],
        0.5,
        Stroke::new(Ink::Hi),
    );
    part.finish()
}

/// One end cap of the body on the belt line, with two screws on its outer face.
fn end_cap(part: &mut Part, side: f32) {
    let inner = 5.0;
    let outer = 7.2;
    let (near, far) = if side > 0.0 {
        (inner, outer)
    } else {
        (-outer, -inner)
    };
    let plan = Plan {
        x: near,
        y: -4.5,
        w: far - near,
        d: 9.0,
        r: 0.6,
    };
    part.slab(&Base::Plan(plan, 2), 2.5, 7.5, 0.3, Finish::paper());
    let face = outer * side;
    for y in [-2.5, 2.5] {
        part.dot(
            [face + side * LIFT, y, 5.0],
            [side, 0.0, 0.0],
            0.45,
            Stroke::new(Ink::Mid).near(),
        );
    }
}
