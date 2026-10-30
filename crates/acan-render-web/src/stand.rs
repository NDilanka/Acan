//! Where a bot hovers beside a machine. A bot sits on the ground beside its machine, clear of every
//! part of it, and turns to face the machine. The first two spots sit behind the machine, one to
//! each side of the default view, so a bot seated there turns its eye toward the default camera.
//! A model is the same for every machine of its kind, so the spots are found once per kind and
//! moved to each machine.

use acan_render::Model;

/// The gap a bot keeps between its reach and the nearest part of its machine, in plan units.
pub const CLEAR: f32 = 3.0;

/// How far a model reaches from its origin in the plan: the farthest of its points, in plan units.
#[must_use]
pub fn reach(model: &Model) -> f32 {
    model
        .meshes
        .iter()
        .flat_map(|mesh| mesh.positions.iter())
        .map(|p| p[0].hypot(p[1]))
        .fold(0.0, f32::max)
}

/// The spots a bot can hover at beside a machine, relative to the machine's centre. `half` is the
/// half footprint of the kind. The spots lie on the two axes, each far enough out that a bot's
/// reach clears every part of the machine by `CLEAR`. The first spot is behind the machine on the
/// left of the default view, and the second behind it on the right.
#[must_use]
pub fn beside(machine: &Model, half: [f32; 2], drone: &Model) -> Vec<[f32; 2]> {
    let [ex, ey] = extent(machine, half);
    let gap = reach(drone) + CLEAR;
    let [x, y] = [ex + gap, ey + gap];
    vec![[-x, 0.0], [0.0, -y], [x, 0.0], [0.0, y]]
}

/// How far a machine reaches from its centre on each axis: its footprint half size, or further
/// where a part of the model sticks out past it.
#[must_use]
pub fn extent(machine: &Model, half: [f32; 2]) -> [f32; 2] {
    machine
        .meshes
        .iter()
        .flat_map(|mesh| mesh.positions.iter())
        .fold(half, |e, p| [e[0].max(p[0].abs()), e[1].max(p[1].abs())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kind::{ALL, PlanFootprint};

    fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
        a[0] * b[0] + a[1] * b[1]
    }

    /// The machine's extent from its centre, and the spots beside it.
    fn spots_of(kind: acan_iso::Kind) -> ([f32; 2], Vec<[f32; 2]>) {
        let machine = acan_iso::model(kind);
        let drone = acan_iso::agent_body();
        let (w, d) = kind.footprint();
        let extent = machine
            .meshes
            .iter()
            .flat_map(|mesh| mesh.positions.iter())
            .fold([w / 2.0, d / 2.0], |e, p| {
                [e[0].max(p[0].abs()), e[1].max(p[1].abs())]
            });
        (extent, beside(&machine, [w / 2.0, d / 2.0], &drone))
    }

    #[test]
    fn every_kind_has_four_spots_whose_reach_clears_the_machine() {
        let reach_of = reach(&acan_iso::agent_body());
        for kind in ALL {
            let (extent, spots) = spots_of(kind);
            assert_eq!(spots.len(), 4, "{kind:?} has four spots");
            for spot in &spots {
                let clear_x = spot[0].abs() - extent[0];
                let clear_y = spot[1].abs() - extent[1];
                assert!(
                    clear_x.max(clear_y) >= reach_of + CLEAR - 1e-3,
                    "{kind:?} spot {spot:?} clears the machine by its reach"
                );
            }
        }
    }

    #[test]
    fn the_first_two_spots_sit_behind_the_machine_either_side_of_the_default_view() {
        let right = [
            std::f32::consts::FRAC_1_SQRT_2,
            -std::f32::consts::FRAC_1_SQRT_2,
        ];
        let toward = [std::f32::consts::FRAC_1_SQRT_2; 2];
        for kind in ALL {
            let (_, spots) = spots_of(kind);
            assert!(dot(spots[0], right) < 0.0 && dot(spots[0], toward) < 0.0);
            assert!(dot(spots[1], right) > 0.0 && dot(spots[1], toward) < 0.0);
        }
    }

    #[test]
    fn two_drones_on_one_machine_do_not_overlap() {
        let reach_of = reach(&acan_iso::agent_body());
        for kind in ALL {
            let (_, spots) = spots_of(kind);
            let apart = (spots[0][0] - spots[1][0]).hypot(spots[0][1] - spots[1][1]);
            assert!(
                apart >= 2.0 * reach_of,
                "{kind:?} seats two drones apart: {apart}"
            );
        }
    }
}
