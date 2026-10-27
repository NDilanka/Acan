//! The machine models. Each kind has one file here. Every model is pure and deterministic, centred on the origin
//! with its base on z = 0.

mod assembler;
// `crate` is a reserved word, so the crate's file cannot be named `crate.rs`.
mod crate_box;
mod gate;
mod intake;
mod splitter;
mod tank;
mod valve;

use crate::model::Model;

/// The machine kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A tall unit with a side panel of lights and four stacked trays.
    Intake,
    /// A tall auth post pair with a turnstile hub, a badge reader and a lintel lamp.
    Gate,
    /// A stack of boards with a gauge, a knob, two dials and a lamp.
    Assembler,
    /// A relational database: a squat cylinder stack with banded seams, a gauge and pipe stubs.
    Tank,
    /// A round hub with four chutes and a crown dial.
    Splitter,
    /// A small belt fitting: a body with a handwheel.
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

/// The model for `kind`. The same kind always gives the same model.
#[must_use]
pub fn model(kind: Kind) -> Model {
    match kind {
        Kind::Intake => intake::build(),
        Kind::Gate => gate::build(),
        Kind::Assembler => assembler::build(),
        Kind::Tank => tank::build(),
        Kind::Splitter => splitter::build(),
        Kind::Valve => valve::build(),
        Kind::Crate => crate_box::build(),
    }
}

/// The footprint of `kind`, from the kit's table.
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
        Kind::Tank => Footprint {
            w: 78.0,
            d: 62.0,
            top: 56.0,
        },
        Kind::Splitter => Footprint {
            w: 86.0,
            d: 86.0,
            top: 34.0,
        },
        Kind::Valve => Footprint {
            w: 18.0,
            d: 18.0,
            top: 16.0,
        },
        Kind::Crate => Footprint {
            w: 40.0,
            d: 40.0,
            top: 30.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::tests::{check_closed, check_outward};

    const ALL: [Kind; 7] = [
        Kind::Intake,
        Kind::Gate,
        Kind::Assembler,
        Kind::Tank,
        Kind::Splitter,
        Kind::Valve,
        Kind::Crate,
    ];

    /// The triangles a model draws at every level of detail.
    fn triangles(model: &Model) -> usize {
        model.meshes.iter().map(|mesh| mesh.indices.len() / 3).sum()
    }

    #[test]
    fn every_model_is_deterministic() {
        for kind in ALL {
            assert_eq!(model(kind), model(kind), "{kind:?}");
        }
    }

    #[test]
    fn every_mesh_is_a_union_of_closed_solids_with_outward_normals() {
        for kind in ALL {
            let built = model(kind);
            for (level, mesh) in built.meshes.iter().enumerate() {
                check_closed(mesh).unwrap_or_else(|e| panic!("{kind:?} level {level}: {e}"));
                check_outward(mesh).unwrap_or_else(|e| panic!("{kind:?} level {level}: {e}"));
                assert_eq!(
                    mesh.positions.len(),
                    mesh.normals.len(),
                    "{kind:?} level {level}"
                );
                assert_eq!(
                    mesh.positions.len(),
                    mesh.surfaces.len(),
                    "{kind:?} level {level}"
                );
            }
        }
    }

    #[test]
    fn bounds_contain_every_vertex_line_and_dot_and_fit_the_footprint() {
        for kind in ALL {
            let built = model(kind);
            let fp = footprint(kind);
            let bounds = built.bounds;
            let inside =
                |p: [f32; 3]| (0..3).all(|a| p[a] >= bounds.min[a] && p[a] <= bounds.max[a]);
            for mesh in &built.meshes {
                assert!(mesh.positions.iter().all(|&p| inside(p)), "{kind:?}");
            }
            for line in &built.lines {
                assert!(line.points.iter().all(|&p| inside(p)), "{kind:?}");
            }
            assert!(bounds.min[2] >= 0.0, "{kind:?} sits on the ground");
            assert!(
                bounds.max[2] <= fp.top + 1e-3,
                "{kind:?} top {} above footprint {}",
                bounds.max[2],
                fp.top
            );
            assert!(
                bounds.min[0] >= -fp.w / 2.0 - 1e-3 && bounds.max[0] <= fp.w / 2.0 + 1e-3,
                "{kind:?} x"
            );
            assert!(
                bounds.min[1] >= -fp.d / 2.0 - 1e-3 && bounds.max[1] <= fp.d / 2.0 + 1e-3,
                "{kind:?} y"
            );
        }
    }

    #[test]
    fn every_model_stays_within_the_budget_at_close() {
        for kind in ALL {
            let built = model(kind);
            let total = triangles(&built);
            println!(
                "{kind:?}: triangles always {}, near {}, close {}, total {total}; lines {}, dots {}",
                built.meshes[0].indices.len() / 3,
                built.meshes[1].indices.len() / 3,
                built.meshes[2].indices.len() / 3,
                built.lines.len(),
                built.dots.len(),
            );
            assert!(total <= 4000, "{kind:?} draws {total} triangles");
        }
    }
}
