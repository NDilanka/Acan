//! Places a model's parts. Floor parts come first. Upright parts follow, back to front by depth,
//! and any part mounted on another is drawn right after it.

use std::cmp::Ordering;

use crate::camera::Camera;
use crate::figure::{Figure, Item};
use crate::pen::Pen;

#[derive(Debug)]
pub(crate) struct Scene {
    cam: Camera,
    floor: Vec<Item>,
    upright: Vec<(f32, Vec<Item>)>,
}

impl Scene {
    pub(crate) fn new() -> Self {
        Self {
            cam: Camera::kit(),
            floor: Vec::new(),
            upright: Vec::new(),
        }
    }

    /// Adds a part that lies on the floor. Floor parts are drawn before every upright part.
    pub(crate) fn floor(&mut self, pen: Pen) {
        self.floor.extend(pen.items);
    }

    /// Adds an upright part centred at `at`. Each mount is a part sitting on this one, placed at its own
    /// centre, and is drawn after it, back to front.
    pub(crate) fn put(&mut self, at: [f32; 3], pen: Pen, mut mounts: Vec<([f32; 3], Pen)>) {
        let cam = self.cam;
        mounts.sort_by(|a, b| depth_order(cam.depth(a.0), cam.depth(b.0)));
        let mut items = pen.items;
        for (_, mount) in mounts {
            items.extend(mount.items);
        }
        self.upright.push((cam.depth(at), items));
    }

    pub(crate) fn finish(mut self) -> Figure {
        self.upright.sort_by(|a, b| depth_order(a.0, b.0));
        let mut items = self.floor;
        for (_, part) in self.upright {
            items.extend(part);
        }
        Figure::from_items(items)
    }
}

fn depth_order(a: f32, b: f32) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::{Fill, Ink};
    use crate::pen::Look;
    use crate::plan::Plan;

    fn box_pen(plan: Plan, z: f32, height: f32, edge: Ink) -> Pen {
        let mut pen = Pen::new();
        pen.solid(&plan.ring(2), z, height, Look::new(edge));
        pen
    }

    fn first_index(items: &[Item], want: impl Fn(&Item) -> bool) -> usize {
        items.iter().position(want).expect("item present")
    }

    #[test]
    fn a_lower_part_is_drawn_before_the_part_stacked_on_it() {
        let plan = Plan {
            x: -5.0,
            y: -5.0,
            w: 10.0,
            d: 10.0,
            r: 1.0,
        };
        let mut scene = Scene::new();
        // The upper box goes in first, so this only passes if depth decides the order.
        scene.put([0.0, 0.0, 9.0], box_pen(plan, 6.0, 6.0, Ink::Hi), vec![]);
        scene.put([0.0, 0.0, 3.0], box_pen(plan, 0.0, 6.0, Ink::Lo), vec![]);
        let figure = scene.finish();
        let faces: Vec<usize> = figure
            .items
            .iter()
            .enumerate()
            .filter(|(_, i)| {
                matches!(
                    i,
                    Item::Fill {
                        tone: Fill::Face,
                        ..
                    }
                )
            })
            .map(|(index, _)| index)
            .collect();
        let [lower_face, upper_face] = faces[..] else {
            panic!("two solids expected")
        };
        let lower_last_line = figure
            .items
            .iter()
            .rposition(|i| matches!(i, Item::Line { tone: Ink::Lo, .. }))
            .unwrap();
        assert!(lower_face < lower_last_line);
        assert!(
            lower_last_line < upper_face,
            "the lower part was not finished before the upper one began"
        );
    }

    #[test]
    fn floor_parts_come_before_upright_parts() {
        let mut scene = Scene::new();
        let mut floor = Pen::new();
        floor.items.push(Item::Fill {
            ring: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            tone: Fill::Halo,
        });
        scene.floor(floor);
        let upright = box_pen(
            Plan {
                x: -2.0,
                y: -2.0,
                w: 4.0,
                d: 4.0,
                r: 0.0,
            },
            0.0,
            2.0,
            Ink::Mid,
        );
        scene.put([0.0, 0.0, 0.0], upright, vec![]);
        let figure = scene.finish();
        assert!(matches!(
            figure.items[0],
            Item::Fill {
                tone: Fill::Halo,
                ..
            }
        ));
    }

    #[test]
    fn mounts_are_drawn_back_to_front_after_their_part() {
        // The far mount is centred at depth -8.8 and the near one at depth 1.0, so the far one must come first.
        let mut scene = Scene::new();
        let near = box_pen(
            Plan {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                r: 0.0,
            },
            0.0,
            1.0,
            Ink::Hi,
        );
        let far = box_pen(
            Plan {
                x: -9.0,
                y: -9.0,
                w: 1.0,
                d: 1.0,
                r: 0.0,
            },
            0.0,
            1.0,
            Ink::Lo,
        );
        let mut base = Pen::new();
        base.items.push(Item::Fill {
            ring: vec![[0.0, 0.0]],
            tone: Fill::Face,
        });
        scene.put(
            [0.0, 0.0, 0.0],
            base,
            vec![([0.0, 0.0, 2.0], near), ([-8.0, -8.0, 2.0], far)],
        );
        let figure = scene.finish();
        let far_at = first_index(&figure.items, |i| {
            matches!(i, Item::Line { tone: Ink::Lo, .. })
        });
        let near_at = first_index(&figure.items, |i| {
            matches!(i, Item::Line { tone: Ink::Hi, .. })
        });
        assert!(
            far_at < near_at,
            "the near mount was drawn before the far one"
        );
    }

    #[test]
    fn the_same_parts_give_the_same_figure() {
        let build = || {
            let mut scene = Scene::new();
            scene.put(
                [0.0, 0.0, 1.0],
                box_pen(
                    Plan {
                        x: 0.0,
                        y: 0.0,
                        w: 3.0,
                        d: 3.0,
                        r: 1.0,
                    },
                    0.0,
                    2.0,
                    Ink::Mid,
                ),
                vec![],
            );
            scene.finish()
        };
        assert_eq!(build(), build());
    }
}
