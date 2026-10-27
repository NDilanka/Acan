//! Solids: a plan ring raised from `z` to `z + height`, drawn the way the kit's `extrude` draws it.
//!
//! The visible sides are shaded by how square they face the key light. Neighbouring sides in the same
//! shade become one fill. The kit draws one fill per side, which looks the same, and merging keeps the
//! item count of a cylinder low.

use crate::camera::Camera;
use crate::figure::{Fill, Item};
use crate::geom::{Point, convex_hull, facing_edges, oriented, runs_of};
use crate::pen::{Look, Pen, Stroke, line_item};

const SHADES: [Fill; 4] = [Fill::Shade0, Fill::Shade1, Fill::Shade2, Fill::Shade3];

impl Pen {
    /// Draws a solid with nothing on its faces.
    pub(crate) fn solid(&mut self, ring: &[Point], z: f32, height: f32, look: Look) {
        self.solid_with(ring, z, height, look, |_| {});
    }

    /// Draws a solid, then `inner` (whatever sits on its faces), then its outline. The kit draws in this order,
    /// so the outline stays on top of what is mounted.
    pub(crate) fn solid_with(
        &mut self,
        ring: &[Point],
        z: f32,
        height: f32,
        look: Look,
        inner: impl FnOnce(&mut Pen),
    ) {
        let (body, edges) = build(&self.cam, ring, z, height, look);
        self.items.extend(body);
        inner(self);
        self.items.extend(edges);
    }

    /// The soft shadow under a footprint: the hull of `ring` at `z` and at `z + height`.
    pub(crate) fn halo(&mut self, ring: &[Point], z: f32, height: f32) {
        let points: Vec<Point> = ring
            .iter()
            .flat_map(|&[x, y]| {
                [
                    self.cam.project([x, y, z + height]),
                    self.cam.project([x, y, z]),
                ]
            })
            .collect();
        self.items.push(Item::Fill {
            ring: convex_hull(&points),
            tone: Fill::Halo,
        });
    }
}

/// The body items (fills, creases, bevel) and the outline items of one solid.
fn build(cam: &Camera, ring: &[Point], z: f32, height: f32, look: Look) -> (Vec<Item>, Vec<Item>) {
    let count = ring.len();
    let (normals, facing) = facing_edges(ring, cam.toward_viewer());
    let top: Vec<Point> = ring
        .iter()
        .map(|&[x, y]| cam.project([x, y, z + height]))
        .collect();
    let bottom: Vec<Point> = ring.iter().map(|&[x, y]| cam.project([x, y, z])).collect();
    let mut body = Vec::new();

    let silhouette: Vec<Point> = top.iter().chain(&bottom).copied().collect();
    body.push(Item::Fill {
        ring: convex_hull(&silhouette),
        tone: Fill::Face,
    });

    if height > 0.0 {
        push_shades(&mut body, cam, &normals, &facing, &top, &bottom);
    }
    body.push(Item::Fill {
        ring: oriented(&top),
        tone: Fill::Top,
    });

    if let Some(crease) = look.crease {
        for run in runs_of(&facing, true) {
            body.push(line_item(
                run.iter().map(|&i| top[i]).collect(),
                false,
                Stroke::new(crease),
            ));
        }
        if look.bevel > 0.0 {
            let inset = inset_top(cam, ring, &normals, look.bevel, z + height);
            for run in runs_of(&facing, true) {
                let last = run.len() - 1;
                let mut points = vec![top[run[0]]];
                points.extend(run[1..last].iter().map(|&i| inset[i]));
                points.push(top[run[last]]);
                body.push(line_item(points, false, Stroke::new(crease).near()));
            }
        }
    }

    let mut edges = Vec::new();
    for run in runs_of(&facing, false) {
        edges.push(line_item(
            run.iter().map(|&i| top[i]).collect(),
            false,
            Stroke::new(look.edge),
        ));
    }
    if height > 0.0 {
        for run in runs_of(&facing, true) {
            edges.push(line_item(
                run.iter().map(|&i| bottom[i]).collect(),
                false,
                Stroke::new(look.edge),
            ));
        }
        for i in 0..count {
            if facing[(i + count - 1) % count] != facing[i] {
                edges.push(line_item(
                    vec![top[i], bottom[i]],
                    false,
                    Stroke::new(look.edge),
                ));
            }
        }
    }
    (body, edges)
}

/// The visible sides, merged into one fill per run of equal shade, drawn shade 0 first.
fn push_shades(
    body: &mut Vec<Item>,
    cam: &Camera,
    normals: &[Point],
    facing: &[bool],
    top: &[Point],
    bottom: &[Point],
) {
    let light = cam.light();
    let shade_of: Vec<usize> = normals
        .iter()
        .map(|n| {
            let lit = (n[0] * light[0] + n[1] * light[1] + 1.0) / 2.0;
            (lit * SHADES.len() as f32)
                .floor()
                .clamp(0.0, (SHADES.len() - 1) as f32) as usize
        })
        .collect();
    let mut strips: [Vec<Vec<Point>>; 4] = Default::default();
    for run in runs_of(facing, true) {
        let edges = &run[..run.len() - 1];
        let mut start = 0;
        while start < edges.len() {
            let shade = shade_of[edges[start]];
            let mut end = start;
            while end + 1 < edges.len() && shade_of[edges[end + 1]] == shade {
                end += 1;
            }
            let verts = &run[start..=end + 1];
            let mut strip: Vec<Point> = verts.iter().map(|&i| top[i]).collect();
            strip.extend(verts.iter().rev().map(|&i| bottom[i]));
            strips[shade].push(oriented(&strip));
            start = end + 1;
        }
    }
    for (shade, rings) in strips.into_iter().enumerate() {
        for ring in rings {
            body.push(Item::Fill {
                ring,
                tone: SHADES[shade],
            });
        }
    }
}

/// Each vertex moved inward along its averaged normal by `bevel`, lifted to `z`.
fn inset_top(cam: &Camera, ring: &[Point], normals: &[Point], bevel: f32, z: f32) -> Vec<Point> {
    let count = ring.len();
    (0..count)
        .map(|i| {
            let a = normals[(i + count - 1) % count];
            let b = normals[i];
            let (mx, my) = (a[0] + b[0], a[1] + b[1]);
            let length = mx.hypot(my);
            let length = if length > 0.0 { length } else { 1.0 };
            let [x, y] = ring[i];
            cam.project([x - mx / length * bevel, y - my / length * bevel, z])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::Ink;

    fn close(actual: Point, expected: Point) -> bool {
        (actual[0] - expected[0]).abs() < 0.02 && (actual[1] - expected[1]).abs() < 0.02
    }

    fn assert_ring(actual: &[Point], expected: &[Point]) {
        assert_eq!(actual.len(), expected.len(), "{actual:?}");
        for (a, e) in actual.iter().zip(expected) {
            assert!(close(*a, *e), "{a:?} != {e:?} in {actual:?}");
        }
    }

    fn slab_r0_ring() -> Vec<Point> {
        crate::plan::Plan {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            d: 6.0,
            r: 0.0,
        }
        .ring(4)
    }

    fn lines(items: &[Item]) -> Vec<(Vec<Point>, bool)> {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Line { points, closed, .. } => Some((points.clone(), *closed)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn slab_top_and_sides_match_the_kit_reference() {
        let mut pen = Pen::new();
        pen.solid(&slab_r0_ring(), 0.0, 5.0, Look::new(Ink::Lo));
        let Item::Fill { ring: top, .. } = pen
            .items
            .iter()
            .find(|i| {
                matches!(
                    i,
                    Item::Fill {
                        tone: Fill::Top,
                        ..
                    }
                )
            })
            .unwrap()
        else {
            panic!("no top")
        };
        assert_ring(
            top,
            &[
                [8.944, -1.005],
                [3.578, 1.679],
                [-5.367, -2.795],
                [0.0, -5.477],
            ],
        );

        let shade0 = pen
            .items
            .iter()
            .find_map(|i| match i {
                Item::Fill {
                    ring,
                    tone: Fill::Shade0,
                } => Some(ring.clone()),
                _ => None,
            })
            .expect("shade 0 side");
        assert_ring(
            &shade0,
            &[
                [8.944, 4.472],
                [3.578, 7.155],
                [3.578, 1.679],
                [8.944, -1.005],
            ],
        );

        let shade3 = pen
            .items
            .iter()
            .find_map(|i| match i {
                Item::Fill {
                    ring,
                    tone: Fill::Shade3,
                } => Some(ring.clone()),
                _ => None,
            })
            .expect("shade 3 side");
        assert_ring(
            &shade3,
            &[
                [3.578, 7.155],
                [-5.367, 2.683],
                [-5.367, -2.795],
                [3.578, 1.679],
            ],
        );
    }

    #[test]
    fn slab_lines_match_the_kit_reference() {
        let mut pen = Pen::new();
        pen.solid(&slab_r0_ring(), 0.0, 5.0, Look::new(Ink::Lo));
        let open: Vec<Vec<Point>> = lines(&pen.items)
            .into_iter()
            .map(|(points, _)| points)
            .collect();
        // Kit order: the crease on the top, the outline's top run, its bottom run, then the two vertical silhouette edges.
        let expected: [Vec<Point>; 5] = [
            vec![[8.944, -1.005], [3.578, 1.679], [-5.367, -2.795]],
            vec![[-5.367, -2.795], [0.0, -5.477], [8.944, -1.005]],
            vec![[8.944, 4.472], [3.578, 7.155], [-5.367, 2.683]],
            vec![[8.944, -1.005], [8.944, 4.472]],
            vec![[-5.367, -2.795], [-5.367, 2.683]],
        ];
        assert_eq!(open.len(), expected.len(), "{open:?}");
        for (actual, want) in open.iter().zip(&expected) {
            assert_ring(actual, want);
        }
    }

    #[test]
    fn cylinder_top_has_one_vertex_per_step() {
        let ring = crate::plan::circle_ring(0.0, 0.0, 9.0, 32);
        let mut pen = Pen::new();
        pen.solid(&ring, 26.0, 3.0, Look::new(Ink::Hi));
        let top = pen
            .items
            .iter()
            .find_map(|i| match i {
                Item::Fill {
                    ring,
                    tone: Fill::Top,
                } => Some(ring.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(top.len(), 32);
        assert!(top.iter().all(|p| p[0].is_finite() && p[1].is_finite()));
    }

    #[test]
    fn top_face_comes_after_its_sides_and_the_outline_comes_last() {
        let mut pen = Pen::new();
        pen.solid(&slab_r0_ring(), 0.0, 5.0, Look::new(Ink::Mid).bevel(0.6));
        let index = |want: fn(&Item) -> bool| pen.items.iter().position(want).unwrap();
        let top = index(|i| {
            matches!(
                i,
                Item::Fill {
                    tone: Fill::Top,
                    ..
                }
            )
        });
        let last_side = pen
            .items
            .iter()
            .rposition(|i| {
                matches!(
                    i,
                    Item::Fill {
                        tone: Fill::Shade0 | Fill::Shade1 | Fill::Shade2 | Fill::Shade3,
                        ..
                    }
                )
            })
            .unwrap();
        let first_edge = index(|i| matches!(i, Item::Line { tone: Ink::Mid, .. }));
        assert!(last_side < top, "a side was drawn after the top");
        assert!(top < first_edge, "the outline was drawn before the top");
    }

    #[test]
    fn inner_items_sit_between_the_body_and_the_outline() {
        let mut pen = Pen::new();
        pen.solid_with(&slab_r0_ring(), 0.0, 5.0, Look::new(Ink::Mid), |pen| {
            pen.on_top(&[[1.0, 1.0], [2.0, 2.0]], 5.0, false, Stroke::new(Ink::Lo));
        });
        let inner = pen
            .items
            .iter()
            .position(|i| matches!(i, Item::Line { tone: Ink::Lo, .. }))
            .unwrap();
        let outline = pen
            .items
            .iter()
            .position(|i| matches!(i, Item::Line { tone: Ink::Mid, .. }))
            .unwrap();
        assert!(inner < outline);
    }
}
