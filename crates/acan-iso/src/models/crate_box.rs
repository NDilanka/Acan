//! The crate: a plain bevelled box with two seam rings and a groove across its lid. It stands for anything not yet
//! recognised.
//!
//! The reference puts a variant letter on the lid. The model has no text, so the lid is left plain.

use crate::mesh::{Base, Finish, Part, Stroke};
use crate::model::{Ink, Model};
use crate::parts;
use crate::plan::Plan;

const HEIGHT: f32 = 26.0;

pub(crate) fn build() -> Model {
    let mut part = Part::new();
    let side = Plan {
        x: -16.0,
        y: -16.0,
        w: 32.0,
        d: 32.0,
        r: 1.5,
    };
    part.slab(&Base::Plan(side, 2), 0.0, HEIGHT, 0.6, Finish::paper());
    for z in [9.0, 18.0] {
        parts::plan_band(&mut part, side, z, 2, Stroke::new(Ink::Lo).near());
    }
    parts::plan_line(
        &mut part,
        side.inset(3.0),
        HEIGHT,
        2,
        Stroke::new(Ink::Faint),
    );
    parts::screws(&mut part, side, 2.5, HEIGHT);
    part.finish()
}
