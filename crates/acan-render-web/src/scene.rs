//! The plant's scene: a flat ribbon for each belt and one instance for each machine, built from
//! the layout and the acan-iso models.

use acan_render::{Aabb, Instance, Model, Ribbon, Scene};

use crate::bends;
use crate::kind::{self, ALL, PlanFootprint};
use crate::layout::{Belt, Layout};
use crate::marks::MarkKinds;

/// The gap between a belt's end and the plinth it meets, in plan units.
const BELT_PORT_GAP: f32 = 1.5;

/// One acan-iso model per kind, in [`kind::ALL`] order, then the marks: an agent's body, the
/// ring round a waiting agent, and the clash pin. [`mark_kinds`] names the last three.
pub fn models() -> Vec<Model> {
    let mut out: Vec<Model> = ALL.iter().map(|&kind| acan_iso::model(kind)).collect();
    out.extend([
        acan_iso::agent_body(),
        acan_iso::waiting_ring(),
        acan_iso::clash_pin(),
    ]);
    out
}

/// The model kinds of the marks, which follow the machine kinds in [`models`].
pub fn mark_kinds() -> MarkKinds {
    let base = ALL.len() as u16;
    MarkKinds {
        body: base,
        ring: base + 1,
        pin: base + 2,
    }
}

/// The plant: a ribbon for each belt and an instance for each machine. `models` is indexed the
/// way [`kind::index`] indexes kinds.
pub fn plant_scene(layout: &Layout, models: &[Model]) -> Scene {
    let ribbons = belt_ribbons(&layout.belts);
    let mut bounds = Aabb::EMPTY;
    for ribbon in &ribbons {
        for &[x, y] in &ribbon.centre {
            bounds.include([x, y, 0.0]);
        }
    }

    let mut instances = Vec::with_capacity(layout.machines.len());
    for placed in &layout.machines {
        let (w, d) = placed.kind.footprint();
        let offset = [placed.x + w / 2.0, placed.y + d / 2.0, 0.0];
        let index = kind::index(placed.kind);
        for corner in models[index].bounds.corners() {
            bounds.include(std::array::from_fn(|axis| offset[axis] + corner[axis]));
        }
        instances.push(Instance {
            offset,
            kind: index as u16,
            tint: 0,
        });
    }
    if bounds.min[0] > bounds.max[0] {
        bounds = Aabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
    }
    Scene {
        ribbons,
        instances,
        bounds,
    }
}

/// Each belt is a flat ribbon on the ground along its rounded path, stopped just short of both
/// plinths. A belt too short to draw is left out.
fn belt_ribbons(belts: &[Belt]) -> Vec<Ribbon> {
    let (paths, widths): (Vec<Vec<[f32; 2]>>, Vec<f32>) = belts
        .iter()
        .filter_map(|belt| {
            Some((
                trimmed(&belt.path, BELT_PORT_GAP)?,
                belt_width(belt.imports),
            ))
        })
        .unzip();
    bends::round_all(&paths)
        .into_iter()
        .zip(widths)
        .map(|(centre, width)| Ribbon { centre, width })
        .collect()
}

/// A belt's width in plan units, a little wider for more imports. The widest belt still leaves
/// a gap to its neighbour at the layout's belt step.
fn belt_width(imports: u32) -> f32 {
    4.0 + imports.min(8) as f32 * 0.3
}

/// The path with `gap` cut off each end, or `None` when nothing is left.
fn trimmed(path: &[[f32; 2]], gap: f32) -> Option<Vec<[f32; 2]>> {
    let [first, second] = [*path.first()?, *path.get(1)?];
    let n = path.len();
    let [before_last, last] = [path[n - 2], path[n - 1]];
    let (head, tail) = (distance(first, second), distance(before_last, last));
    if head <= gap || tail <= gap || (n == 2 && head <= 2.0 * gap) {
        return None;
    }
    let mut out = path.to_vec();
    out[0] = toward(first, second, gap / head);
    out[n - 1] = toward(last, before_last, gap / tail);
    Some(out)
}

/// The point a fraction `t` of the way from `from` to `to`.
fn toward(from: [f32; 2], to: [f32; 2], t: f32) -> [f32; 2] {
    [
        from[0] + (to[0] - from[0]) * t,
        from[1] + (to[1] - from[1]) * t,
    ]
}

fn distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    let v = [b[0] - a[0], b[1] - a[1]];
    v[0].hypot(v[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kind::Kind;
    use crate::layout::layout;
    use crate::plant::{from_scan_json, stress};

    const FIXTURE: &str = include_str!("../www/fixtures/demo-plant.json");

    #[test]
    fn models_stand_on_the_ground_inside_their_footprint() {
        for (kind, model) in ALL.into_iter().zip(models()) {
            let (w, d) = kind.footprint();
            let b = model.bounds;
            assert!(b.min[2].abs() < 1e-4, "{kind:?} floats above z = 0");
            assert!(b.max[2] > 6.0, "{kind:?} has no body");
            assert!(
                b.min[0] >= -w / 2.0 - 1e-3 && b.max[0] <= w / 2.0 + 1e-3,
                "{kind:?} width"
            );
            assert!(
                b.min[1] >= -d / 2.0 - 1e-3 && b.max[1] <= d / 2.0 + 1e-3,
                "{kind:?} depth"
            );
        }
    }

    #[test]
    fn the_scene_places_each_machine_at_its_footprint_centre() {
        let plant = from_scan_json(FIXTURE).expect("fixture parses");
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let scene = plant_scene(&laid, &models());
        assert_eq!(scene.instances.len(), plant.machines.len());
        for (placed, inst) in laid.machines.iter().zip(&scene.instances) {
            let (w, d) = placed.kind.footprint();
            assert_eq!(inst.offset, [placed.x + w / 2.0, placed.y + d / 2.0, 0.0]);
            assert_eq!(usize::from(inst.kind), kind::index(placed.kind));
        }
        assert_eq!(scene.ribbons.len(), laid.belts.len());
    }

    #[test]
    fn the_scene_bounds_hold_every_machine_and_belt() {
        let plant = stress(200);
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let scene = plant_scene(&laid, &models());
        for placed in &laid.machines {
            let f = placed.footprint();
            assert!(f[0] >= scene.bounds.min[0] && f[2] <= scene.bounds.max[0]);
            assert!(f[1] >= scene.bounds.min[1] && f[3] <= scene.bounds.max[1]);
        }
        for ribbon in &scene.ribbons {
            for p in &ribbon.centre {
                for (axis, value) in p.iter().enumerate() {
                    assert!(*value >= scene.bounds.min[axis] - 1e-3);
                    assert!(*value <= scene.bounds.max[axis] + 1e-3);
                }
            }
        }
    }

    #[test]
    fn no_belt_reaches_under_a_plinth_across_its_whole_width() {
        for plant in [
            from_scan_json(FIXTURE).expect("fixture parses"),
            stress(200),
        ] {
            let (kinds, edges) = plant.layout_input();
            let laid = layout(&kinds, &edges);
            for ribbon in &plant_scene(&laid, &models()).ribbons {
                let c = &ribbon.centre;
                for i in 0..c.len() {
                    let next = c[(i + 1).min(c.len() - 1)];
                    let prev = c[i.saturating_sub(1)];
                    let d = [next[0] - prev[0], next[1] - prev[1]];
                    let len = d[0].hypot(d[1]).max(1e-6);
                    let across = [-d[1] / len, d[0] / len];
                    for s in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                        let k = 0.5 * ribbon.width * s;
                        let p = [c[i][0] + across[0] * k, c[i][1] + across[1] * k];
                        for m in &laid.machines {
                            let f = m.footprint();
                            assert!(
                                !(p[0] > f[0] && p[0] < f[2] && p[1] > f[1] && p[1] < f[3]),
                                "{p:?} of a belt lies under the plinth at {f:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_belt_stops_just_short_of_both_plates() {
        let belt = Belt {
            from: 0,
            to: 1,
            imports: 1,
            path: vec![[0.0, 0.0], [100.0, 0.0]],
        };
        let ribbons = belt_ribbons(std::slice::from_ref(&belt));
        let ribbon = ribbons.first().expect("a long belt draws");
        assert_eq!(
            ribbon.centre,
            vec![[BELT_PORT_GAP, 0.0], [100.0 - BELT_PORT_GAP, 0.0]]
        );
        assert_eq!(ribbon.width, belt_width(1));
        let stub = Belt {
            path: vec![[0.0, 0.0], [2.0, 0.0]],
            ..belt
        };
        assert!(
            belt_ribbons(&[stub]).is_empty(),
            "a belt shorter than its gaps draws nothing"
        );
    }

    #[test]
    fn belts_are_narrow_next_to_the_machines() {
        let (w, d) = Kind::Assembler.footprint();
        assert!(
            belt_width(8) * 10.0 < w.min(d),
            "the widest belt stays under a tenth of a plate"
        );
        assert!(belt_width(1) < belt_width(8));
    }
}
