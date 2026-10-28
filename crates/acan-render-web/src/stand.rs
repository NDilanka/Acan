//! Where a bot can stand on a machine's plinth top. The spots come from the machine's own model,
//! so no bot stands inside a part that rises above the plinth. A model is the same for every
//! machine of its kind, so the spots are found once per kind and moved to each machine.

use acan_iso::PLINTH_TOP;
use acan_render::Model;

/// The clearances a bot keeps from a part that rises above the plinth, in plan units, most room
/// first. A kind that leaves no spot at the first keeps closer to its parts, down to the last.
const CLEARANCES: [f32; 4] = [4.5, 2.0, 1.0, 0.5];
/// The least clearance any spot keeps, so the tests can state it.
pub const MIN_CLEAR: f32 = 0.5;
/// Gap between a plinth edge and the outermost candidate spot, in plan units.
const INSET: f32 = 1.5;
/// Spacing of the candidate spots, in plan units.
const STEP: f32 = 2.5;
/// How many spots a kind keeps, front first. Agents seat on the first few, so more add nothing.
const KEEP: usize = 32;

/// The free spots on a plinth top of a model placed at `(0, 0)`, front first. `half` is the half
/// footprint of the kind, and `front` the direction the default view stands in, so the spots
/// nearest the viewer come first and the nearest the centre among equals. Empty when no spot is
/// free even at the least clearance.
#[must_use]
pub fn spots(model: &Model, half: [f32; 2], front: [f32; 2]) -> Vec<[f32; 2]> {
    let mut parts: Vec<[[f32; 2]; 3]> = Vec::new();
    for mesh in &model.meshes {
        for tri in mesh.indices.chunks_exact(3) {
            let corners = [tri[0], tri[1], tri[2]].map(|i| mesh.positions[i as usize]);
            if corners.iter().any(|p| p[2] > PLINTH_TOP + 1e-3) {
                parts.push(corners.map(|p| [p[0], p[1]]));
            }
        }
    }
    for clear in CLEARANCES {
        let found = grid_spots(&parts, half, front, clear);
        if !found.is_empty() {
            return found;
        }
    }
    Vec::new()
}

/// The grid points on the plinth top that keep `clear` from every part, front first.
fn grid_spots(
    parts: &[[[f32; 2]; 3]],
    half: [f32; 2],
    front: [f32; 2],
    clear: f32,
) -> Vec<[f32; 2]> {
    let nx = ((half[0] - INSET) / STEP).floor().max(0.0) as i32;
    let ny = ((half[1] - INSET) / STEP).floor().max(0.0) as i32;
    let mut free: Vec<([f32; 2], f32, f32)> = Vec::new();
    for i in -nx..=nx {
        for j in -ny..=ny {
            let p = [i as f32 * STEP, j as f32 * STEP];
            if parts.iter().any(|tri| distance_to_triangle(p, tri) < clear) {
                continue;
            }
            let along = p[0] * front[0] + p[1] * front[1];
            free.push((p, along, p[0].hypot(p[1])));
        }
    }
    free.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.2.total_cmp(&b.2)));
    free.into_iter().take(KEEP).map(|(p, _, _)| p).collect()
}

fn distance_to_triangle(p: [f32; 2], tri: &[[f32; 2]; 3]) -> f32 {
    let side =
        |a: [f32; 2], b: [f32; 2]| (p[0] - b[0]) * (a[1] - b[1]) - (a[0] - b[0]) * (p[1] - b[1]);
    let (d0, d1, d2) = (
        side(tri[0], tri[1]),
        side(tri[1], tri[2]),
        side(tri[2], tri[0]),
    );
    let negative = d0 < 0.0 || d1 < 0.0 || d2 < 0.0;
    let positive = d0 > 0.0 || d1 > 0.0 || d2 > 0.0;
    if !(negative && positive) {
        return 0.0;
    }
    (0..3)
        .map(|k| distance_to_segment(p, tri[k], tri[(k + 1) % 3]))
        .fold(f32::INFINITY, f32::min)
}

fn distance_to_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if len2 > 0.0 {
        (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let q = [a[0] + ab[0] * t, a[1] + ab[1] * t];
    (p[0] - q[0]).hypot(p[1] - q[1])
}

#[cfg(test)]
mod tests {
    use acan_iso::Kind;

    use super::*;
    use crate::kind::{ALL, PlanFootprint};

    #[test]
    fn every_kind_has_spots_that_clear_its_parts_and_face_the_front_first() {
        let front = [std::f32::consts::FRAC_1_SQRT_2; 2];
        for kind in ALL {
            let model = acan_iso::model(kind);
            let (w, d) = kind.footprint();
            let found = spots(&model, [w / 2.0, d / 2.0], front);
            assert!(!found.is_empty(), "{kind:?} has a spot for a bot");
            let mut parts: Vec<[[f32; 2]; 3]> = Vec::new();
            for mesh in &model.meshes {
                for tri in mesh.indices.chunks_exact(3) {
                    let corners = [tri[0], tri[1], tri[2]].map(|i| mesh.positions[i as usize]);
                    if corners.iter().any(|p| p[2] > PLINTH_TOP + 1e-3) {
                        parts.push(corners.map(|p| [p[0], p[1]]));
                    }
                }
            }
            for spot in &found {
                assert!(
                    parts
                        .iter()
                        .all(|tri| distance_to_triangle(*spot, tri) >= MIN_CLEAR - 1e-3),
                    "{kind:?} spot {spot:?} clears every part"
                );
                assert!(spot[0].abs() <= w / 2.0 && spot[1].abs() <= d / 2.0);
            }
            let along = |p: &[f32; 2]| p[0] * front[0] + p[1] * front[1];
            assert!(
                found
                    .windows(2)
                    .all(|pair| along(&pair[0]) >= along(&pair[1]) - 1e-4),
                "{kind:?} lists the front first"
            );
        }
    }

    #[test]
    fn a_crate_has_a_spot_at_its_front_corner_not_under_its_lid() {
        let model = acan_iso::model(Kind::Crate);
        let (w, d) = Kind::Crate.footprint();
        let found = spots(&model, [w / 2.0, d / 2.0], [0.0, 1.0]);
        assert!(found[0][1] > 0.0, "the front spot is on the front side");
    }
}
