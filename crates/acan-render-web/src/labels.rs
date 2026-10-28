//! Where machine names go on screen. A label hangs under its plate's front edge, fades with the
//! plant's fog, and is left out when its machine is too small on screen to name or when a
//! nearer machine hides the spot. The page then drops any label that collides with one placed
//! before it, so the order here is the priority: larger on screen first.

use acan_render::camera::Projection;
use acan_render::{Aabb, Camera};

/// A machine named at rest needs at least this many CSS pixels of plate width on screen.
pub const REST_MIN_CSS_PX: f32 = 18.0;
/// With Alt held every machine may be named, but only once its plate is this wide on screen.
pub const ALL_MIN_CSS_PX: f32 = 34.0;
/// The average advance of one character of a label, in CSS pixels, for its width before the
/// page measures it.
pub const CHAR_CSS_PX: f32 = 6.5;
/// From the plate's front edge down to the middle of the label, in CSS pixels. Matches the
/// label's offset and line height in `index.html`.
pub const LABEL_DROP_CSS_PX: f32 = 13.0;
/// The label's bottom edge sits this many `LABEL_DROP_CSS_PX` below the plate's front edge.
const LABEL_BOTTOM_DROPS: f32 = 1.6;
/// Labels fainter than this are left out.
const MIN_OPACITY: f32 = 0.05;
/// A label may sit this far outside the viewport, in device pixels, and still be placed.
const VIEW_MARGIN_PX: f32 = 40.0;

/// A label that may be shown, in device pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelSpot {
    pub index: usize,
    /// The plate's front edge, under the plate's centre.
    pub screen: [f32; 2],
    pub opacity: f32,
    /// The plate's width on screen.
    pub size_px: f32,
}

/// The labels of `candidates` that may show, most important first. `footprints` holds each
/// machine's plan rectangle `[min x, min y, max x, max y]`. `occluders` are world boxes, each
/// with the machine it belongs to, that can hide another machine's label.
/// `half_widths` is each machine's label half width and `drop_px` the distance from the front
/// edge to the label's middle, both in device pixels.
pub fn place(
    camera: &Camera,
    footprints: &[[f32; 4]],
    occluders: &[(usize, Aabb)],
    half_widths: &[f32],
    candidates: &[usize],
    min_px: f32,
    drop_px: f32,
) -> Vec<LabelSpot> {
    let rects: Vec<Option<[f32; 4]>> = occluders
        .iter()
        .map(|(_, b)| screen_rect(camera, b))
        .collect();
    let [vw, vh] = camera.viewport();
    let mut spots: Vec<LabelSpot> = candidates
        .iter()
        .filter_map(|&index| {
            let f = *footprints.get(index)?;
            let corners =
                [[f[0], f[1]], [f[2], f[1]], [f[2], f[3]], [f[0], f[3]]].map(|[x, y]| [x, y, 0.0]);
            if !corners.iter().all(|&c| camera.in_front(c)) {
                return None;
            }
            let screen = corners.map(|c| camera.world_to_screen(c));
            let xs = screen.map(|s| s[0]);
            let size_px = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                - xs.iter().copied().fold(f32::INFINITY, f32::min);
            if size_px < min_px {
                return None;
            }
            let front = (0..4).max_by(|&a, &b| screen[a][1].total_cmp(&screen[b][1]))?;
            let centre = [(f[0] + f[2]) * 0.5, (f[1] + f[3]) * 0.5, 0.0];
            let anchor = [camera.world_to_screen(centre)[0], screen[front][1]];
            let inside = |v: f32, hi: f32| v >= -VIEW_MARGIN_PX && v <= hi + VIEW_MARGIN_PX;
            if !inside(anchor[0], vw) || !inside(anchor[1], vh) {
                return None;
            }
            let opacity = 1.0 - camera.fog_at(corners[front]);
            if opacity < MIN_OPACITY {
                return None;
            }
            let out = unit2([corners[front][0] - centre[0], corners[front][1] - centre[1]]);
            let edge = [corners[front][0] + out[0], corners[front][1] + out[1], 0.5];
            // The ground under the label's middle row and its bottom, at five points across, so a nearer machine that covers any part of the text hides it.
            let half = half_widths.get(index).copied().unwrap_or(0.0);
            let under = [drop_px, drop_px * LABEL_BOTTOM_DROPS]
                .into_iter()
                .flat_map(|drop| {
                    [-1.0, -0.5, 0.0, 0.5, 1.0].map(|t| [anchor[0] + t * half, anchor[1] + drop])
                })
                .filter_map(|px| camera.ground_at(px))
                .map(|[x, y]| [x, y, 0.2]);
            let hidden = std::iter::once(edge)
                .chain(under)
                .any(|p| hidden_by_other(camera, p, index, occluders, &rects));
            (!hidden).then_some(LabelSpot {
                index,
                screen: anchor,
                opacity,
                size_px,
            })
        })
        .collect();
    spots.sort_by(|a, b| b.size_px.total_cmp(&a.size_px).then(a.index.cmp(&b.index)));
    spots
}

/// Whether a box of a machine other than `own` lies between the eye and `p`.
fn hidden_by_other(
    camera: &Camera,
    p: [f32; 3],
    own: usize,
    occluders: &[(usize, Aabb)],
    rects: &[Option<[f32; 4]>],
) -> bool {
    let s = camera.world_to_screen(p);
    let origin = match camera.projection() {
        Projection::Perspective => camera.eye(),
        Projection::Orthographic => {
            let t = camera.basis().toward;
            let reach = 4.0 * camera.eye_distance();
            std::array::from_fn(|axis| p[axis] + t[axis] * reach)
        }
    };
    occluders.iter().zip(rects).any(|(&(owner, ref b), rect)| {
        if owner == own {
            return false;
        }
        let maybe =
            rect.is_none_or(|r| s[0] >= r[0] && s[0] <= r[2] && s[1] >= r[1] && s[1] <= r[3]);
        maybe && segment_hits(origin, p, b)
    })
}

/// The screen rectangle around a box's corners, or `None` when part of it is behind the eye.
fn screen_rect(camera: &Camera, b: &Aabb) -> Option<[f32; 4]> {
    let mut r: [f32; 4] = [
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for c in b.corners() {
        if !camera.in_front(c) {
            return None;
        }
        let s = camera.world_to_screen(c);
        r = [
            r[0].min(s[0]),
            r[1].min(s[1]),
            r[2].max(s[0]),
            r[3].max(s[1]),
        ];
    }
    Some(r)
}

/// Whether the segment from `a` to `b` passes through the box, short of `b` itself.
fn segment_hits(a: [f32; 3], b: [f32; 3], bx: &Aabb) -> bool {
    let (mut t0, mut t1) = (0.0_f32, 1.0_f32 - 1e-4);
    for axis in 0..3 {
        let d = b[axis] - a[axis];
        if d.abs() < 1e-9 {
            if a[axis] < bx.min[axis] || a[axis] > bx.max[axis] {
                return false;
            }
            continue;
        }
        let (mut near, mut far) = ((bx.min[axis] - a[axis]) / d, (bx.max[axis] - a[axis]) / d);
        if near > far {
            std::mem::swap(&mut near, &mut far);
        }
        t0 = t0.max(near);
        t1 = t1.min(far);
        if t0 > t1 {
            return false;
        }
    }
    true
}

fn unit2(v: [f32; 2]) -> [f32; 2] {
    let len = v[0].hypot(v[1]).max(1e-6);
    [v[0] / len, v[1] / len]
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 96.0;
    const D: f32 = 86.0;
    const H: f32 = 40.0;

    fn machine(x: f32, y: f32) -> ([f32; 4], Aabb) {
        (
            [x, y, x + W, y + D],
            Aabb {
                min: [x, y, 0.0],
                max: [x + W, y + D, H],
            },
        )
    }

    fn camera_on(machines: &[([f32; 4], Aabb)], azimuth: f32, elevation: f32) -> Camera {
        let mut bounds = Aabb::EMPTY;
        for (_, b) in machines {
            bounds.include(b.min);
            bounds.include(b.max);
        }
        let mut camera = Camera::new([1440.0, 900.0]);
        camera.set_angles(azimuth, elevation);
        camera.fit(&bounds, 24.0);
        camera
    }

    fn split(machines: &[([f32; 4], Aabb)]) -> (Vec<[f32; 4]>, Vec<(usize, Aabb)>) {
        let footprints = machines.iter().map(|(f, _)| *f).collect();
        let occluders = machines.iter().map(|(_, b)| *b).enumerate().collect();
        (footprints, occluders)
    }

    fn placed(machines: &[([f32; 4], Aabb)], camera: &Camera) -> Vec<usize> {
        let (footprints, occluders) = split(machines);
        let all: Vec<usize> = (0..machines.len()).collect();
        place(
            camera,
            &footprints,
            &occluders,
            &[],
            &all,
            10.0,
            LABEL_DROP_CSS_PX,
        )
        .into_iter()
        .map(|s| s.index)
        .collect()
    }

    #[test]
    fn a_label_behind_a_nearer_machine_at_low_tilt_is_hidden() {
        // The camera looks along -x at azimuth 0, so the machine at larger x is in front.
        let machines = [machine(0.0, 0.0), machine(130.0, 0.0)];
        let camera = camera_on(&machines, 0.0, 8.0);
        assert_eq!(
            placed(&machines, &camera),
            vec![1],
            "only the front machine is named"
        );
    }

    #[test]
    fn a_label_whose_end_runs_onto_a_nearer_machine_is_hidden() {
        // Machine 1 stands in front of machine 0 and off to one side, so only a wide label
        // reaches it.
        let machines = [machine(0.0, 0.0), machine(130.0, 110.0)];
        let camera = camera_on(&machines, 0.0, 8.0);
        let (footprints, occluders) = split(&machines);
        let shown = |half: f32| {
            place(
                &camera,
                &footprints,
                &occluders,
                &[half, half],
                &[0],
                10.0,
                LABEL_DROP_CSS_PX,
            )
            .len()
        };
        let centre_x = |f: [f32; 4]| {
            let c = [(f[0] + f[2]) * 0.5, (f[1] + f[3]) * 0.5, 0.0];
            camera.world_to_screen(c)[0]
        };
        let reach = (centre_x(footprints[1]) - centre_x(footprints[0])).abs();
        assert_eq!(
            shown(0.0),
            1,
            "a point label beside the nearer machine shows"
        );
        assert_eq!(shown(reach), 0, "a label wide enough to reach it is hidden");
    }

    #[test]
    fn side_by_side_machines_both_keep_their_labels() {
        let machines = [machine(0.0, 0.0), machine(0.0, 200.0)];
        let camera = camera_on(&machines, 0.0, 30.0);
        let mut shown = placed(&machines, &camera);
        shown.sort();
        assert_eq!(shown, vec![0, 1]);
    }

    #[test]
    fn the_larger_machine_on_screen_comes_first() {
        let machines = [machine(0.0, 0.0), machine(400.0, 0.0)];
        let camera = camera_on(&machines, 0.0, 30.0);
        let shown = placed(&machines, &camera);
        assert_eq!(
            shown.first(),
            Some(&1),
            "the nearer machine leads: {shown:?}"
        );
    }

    #[test]
    fn a_machine_too_small_on_screen_is_not_named() {
        let machines = [machine(0.0, 0.0)];
        let camera = camera_on(&machines, 45.0, 30.0);
        let (footprints, occluders) = split(&machines);
        let spots = place(
            &camera,
            &footprints,
            &occluders,
            &[],
            &[0],
            1e6,
            LABEL_DROP_CSS_PX,
        );
        assert!(spots.is_empty());
    }

    #[test]
    fn the_anchor_is_the_front_edge_of_the_plate() {
        let machines = [machine(0.0, 0.0)];
        let camera = camera_on(&machines, 45.0, 30.0);
        let (footprints, occluders) = split(&machines);
        let spot = place(
            &camera,
            &footprints,
            &occluders,
            &[],
            &[0],
            1.0,
            LABEL_DROP_CSS_PX,
        )[0];
        let f = footprints[0];
        let lowest = [[f[0], f[1]], [f[2], f[1]], [f[2], f[3]], [f[0], f[3]]]
            .map(|[x, y]| camera.world_to_screen([x, y, 0.0])[1])
            .into_iter()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((spot.screen[1] - lowest).abs() < 1e-3);
        assert!(spot.opacity > 0.9, "a fitted plant is not fogged");
    }

    #[test]
    fn a_segment_through_a_box_hits_and_one_beside_it_misses() {
        let b = Aabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        assert!(segment_hits([-1.0, 0.5, 0.5], [2.0, 0.5, 0.5], &b));
        assert!(!segment_hits([-1.0, 2.0, 0.5], [2.0, 2.0, 0.5], &b));
        assert!(
            !segment_hits([-3.0, 0.5, 0.5], [-1.0, 0.5, 0.5], &b),
            "stops short"
        );
    }
}
