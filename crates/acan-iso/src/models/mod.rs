//! The machine models. Each kind with a drawing has one file here. A kind without one draws as the crate.

mod assembler;
// `crate` is a reserved word, so the crate's file cannot be named `crate.rs`.
mod crate_box;
mod gate;
mod intake;
mod splitter;

use crate::figure::Figure;

/// The machine kinds. `Tank` and `Valve` have no drawing yet, so they draw as the crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A tall unit with a side panel of lights and four stacked trays.
    Intake,
    /// A tall post with a centre hub, a side block and a head lamp.
    Gate,
    /// A stack of boards with a gauge, a knob, two dials and a lamp.
    Assembler,
    /// A vessel. Not drawn yet; draws as the crate.
    Tank,
    /// A round hub with four chutes and a crown dial.
    Splitter,
    /// A valve. Not drawn yet; draws as the crate.
    Valve,
    /// A plain box, and what every kind without a drawing draws as.
    Crate,
}

/// A model's size in plan units. The model is centred on the origin, so its plan runs from `-w / 2` to `w / 2`
/// and from `-d / 2` to `d / 2`, and it stands `top` high.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Footprint {
    /// Width along x.
    pub w: f32,
    /// Depth along y.
    pub d: f32,
    /// Height of the model's top above the floor.
    pub top: f32,
}

/// The draw list for `kind`. The same kind always gives the same figure.
#[must_use]
pub fn model(kind: Kind) -> Figure {
    match kind {
        Kind::Intake => intake::build(),
        Kind::Gate => gate::build(),
        Kind::Assembler => assembler::build(),
        Kind::Splitter => splitter::build(),
        Kind::Tank | Kind::Valve | Kind::Crate => crate_box::build(),
    }
}

/// The footprint of `kind`, from the kit's table. A kind that draws as the crate reports the crate's footprint.
#[must_use]
pub fn footprint(kind: Kind) -> Footprint {
    match kind {
        Kind::Intake => Footprint {
            w: 50.0,
            d: 96.0,
            top: 30.0,
        },
        Kind::Gate => Footprint {
            w: 44.0,
            d: 74.0,
            top: 40.0,
        },
        Kind::Assembler => Footprint {
            w: 96.0,
            d: 86.0,
            top: 66.0,
        },
        Kind::Splitter => Footprint {
            w: 86.0,
            d: 86.0,
            top: 34.0,
        },
        Kind::Tank | Kind::Valve | Kind::Crate => Footprint {
            w: 40.0,
            d: 40.0,
            top: 30.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::{Detail, Fill, Item};

    const ALL: [Kind; 7] = [
        Kind::Intake,
        Kind::Gate,
        Kind::Assembler,
        Kind::Tank,
        Kind::Splitter,
        Kind::Valve,
        Kind::Crate,
    ];

    fn points(item: &Item) -> Vec<[f32; 2]> {
        match item {
            Item::Fill { ring, .. } => ring.clone(),
            Item::Line { points, .. } | Item::Dots { points, .. } => points.clone(),
        }
    }

    /// Items drawn at `level` or coarser: fills always, lines and dots when their detail is at or below `level`.
    fn visible(figure: &Figure, level: Detail) -> usize {
        figure
            .items
            .iter()
            .filter(|item| match item {
                Item::Fill { .. } => true,
                Item::Line { detail, .. } | Item::Dots { detail, .. } => *detail <= level,
            })
            .count()
    }

    #[test]
    fn every_model_is_deterministic() {
        for kind in ALL {
            assert_eq!(model(kind), model(kind), "{kind:?}");
        }
    }

    #[test]
    fn every_model_has_finite_bounds_that_contain_all_its_items() {
        for kind in ALL {
            let figure = model(kind);
            assert!(!figure.items.is_empty(), "{kind:?} is empty");
            let [x0, y0, x1, y1] = figure.bounds;
            assert!(
                [x0, y0, x1, y1].iter().all(|v| v.is_finite()),
                "{kind:?} bounds {:?}",
                figure.bounds
            );
            assert!(x0 < x1 && y0 < y1, "{kind:?} bounds {:?}", figure.bounds);
            for item in &figure.items {
                for [x, y] in points(item) {
                    assert!(x.is_finite() && y.is_finite(), "{kind:?} non-finite point");
                    assert!(
                        x >= x0 && x <= x1 && y >= y0 && y <= y1,
                        "{kind:?}: {x},{y} outside {:?}",
                        figure.bounds
                    );
                }
            }
        }
    }

    #[test]
    fn every_solid_draws_its_top_after_its_sides() {
        // A solid starts with its face fill. Within it, no side may come after the top.
        for kind in ALL {
            let mut top_seen = false;
            for item in &model(kind).items {
                match item {
                    Item::Fill {
                        tone: Fill::Face, ..
                    } => top_seen = false,
                    Item::Fill {
                        tone: Fill::Top, ..
                    } => top_seen = true,
                    Item::Fill {
                        tone: Fill::Shade0 | Fill::Shade1 | Fill::Shade2 | Fill::Shade3,
                        ..
                    } => {
                        assert!(!top_seen, "{kind:?}: a side was drawn after its top");
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn the_assembler_fits_the_renderer_budget_at_close() {
        let figure = model(Kind::Assembler);
        let total = visible(&figure, Detail::Close);
        assert!(total <= 600, "{total} items at Close");
    }

    #[test]
    fn every_model_fits_the_renderer_budget_at_close() {
        for kind in ALL {
            let total = visible(&model(kind), Detail::Close);
            assert!(total <= 600, "{kind:?}: {total} items at Close");
        }
    }

    #[test]
    fn kinds_without_a_drawing_draw_and_measure_as_the_crate() {
        assert_eq!(model(Kind::Tank), model(Kind::Crate));
        assert_eq!(model(Kind::Valve), model(Kind::Crate));
        assert_eq!(footprint(Kind::Tank), footprint(Kind::Crate));
        assert_eq!(footprint(Kind::Valve), footprint(Kind::Crate));
    }

    #[test]
    fn footprints_match_the_kit_table() {
        assert_eq!(
            footprint(Kind::Assembler),
            Footprint {
                w: 96.0,
                d: 86.0,
                top: 66.0
            }
        );
        assert_eq!(
            footprint(Kind::Crate),
            Footprint {
                w: 40.0,
                d: 40.0,
                top: 30.0
            }
        );
        assert_eq!(
            footprint(Kind::Intake),
            Footprint {
                w: 50.0,
                d: 96.0,
                top: 30.0
            }
        );
        assert_eq!(
            footprint(Kind::Gate),
            Footprint {
                w: 44.0,
                d: 74.0,
                top: 40.0
            }
        );
        assert_eq!(
            footprint(Kind::Splitter),
            Footprint {
                w: 86.0,
                d: 86.0,
                top: 34.0
            }
        );
    }
}
