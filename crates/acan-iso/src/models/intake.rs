//! The intake: a tall unit with a side panel of indicator dots and a lamp, and four stacked trays.

use crate::mesh::{Base, Finish, LIFT, Part, Stroke};
use crate::model::{Ink, Model};
use crate::parts;
use crate::plan::Plan;

/// Height of the panel rail's top, and of the trays' tops.
const RAIL_TOP: f32 = 28.0;
const TRAY_TOP: f32 = 10.0;

pub(crate) fn build() -> Model {
    let mut part = Part::new();
    parts::plinth(&mut part, 50.0, 96.0);
    panel(&mut part);
    for row in 0..4 {
        tray(&mut part, -38.0 + row as f32 * 20.0);
    }
    part.finish()
}

/// The side panel: a tall rail with four indicator dots and a lamp at its top end.
fn panel(part: &mut Part) {
    let rail = Plan {
        x: -22.0,
        y: -44.0,
        w: 6.0,
        d: 88.0,
        r: 1.5,
    };
    part.slab(&Base::Plan(rail, 3), 3.0, RAIL_TOP, 0.6, Finish::paper());
    for y in [-30.0, -10.0, 10.0, 30.0] {
        part.dot(
            [-19.0, y, RAIL_TOP + LIFT],
            [0.0, 0.0, 1.0],
            0.45,
            Stroke::new(Ink::Mid).near(),
        );
    }
    parts::lamp(part, [-19.0, 40.0], RAIL_TOP);
}

/// One tray: a bevelled slab with a groove across its top and three lit slots.
fn tray(part: &mut Part, y: f32) {
    let plan = Plan {
        x: -16.0,
        y,
        w: 34.0,
        d: 14.0,
        r: 1.5,
    };
    part.slab(&Base::Plan(plan, 2), 3.0, TRAY_TOP, 0.4, Finish::paper());
    let groove_y = y + 7.0;
    part.line(
        &[
            [-14.0, groove_y, TRAY_TOP + LIFT],
            [16.0, groove_y, TRAY_TOP + LIFT],
        ],
        false,
        Stroke::new(Ink::Faint),
    );
    for x in [-8.0, 2.0, 12.0] {
        part.dot(
            [x, groove_y, TRAY_TOP + LIFT],
            [0.0, 0.0, 1.0],
            0.9,
            Stroke::new(Ink::Hi).near(),
        );
    }
}
