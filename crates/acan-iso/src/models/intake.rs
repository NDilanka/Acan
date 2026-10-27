//! The intake: a tall unit with a side panel of indicator dots and a lamp, and four stacked trays.
//!
//! Drawn from the reference explore's intake at zero explode.

use crate::figure::{Detail, Figure, Ink};
use crate::pen::{Look, Pen, Stroke};
use crate::plan::Plan;
use crate::scene::Scene;

pub(crate) fn build() -> Figure {
    let mut scene = Scene::new();

    let mut floor = Pen::new();
    floor.plinth(0.0, 0.0, 50.0, 96.0);
    scene.floor(floor);

    let mut panel = Pen::new();
    let rail = Plan {
        x: -22.0,
        y: -44.0,
        w: 6.0,
        d: 88.0,
        r: 1.5,
    };
    panel.solid(&rail.ring(3), 4.0, 24.0, Look::new(Ink::Mid).bevel(0.6));
    let lights = [-30.0, -10.0, 10.0, 30.0].map(|y| [-19.0, y, 28.0]);
    panel.dots(&lights, 0.45, Ink::Mid, Detail::Near);
    panel.lamp([-19.0, 40.0], 28.0);
    scene.put([-19.0, 0.0, 16.0], panel, vec![]);

    for row in 0..4 {
        let y = -38.0 + row as f32 * 20.0;
        let tray = Plan {
            x: -16.0,
            y,
            w: 34.0,
            d: 14.0,
            r: 1.5,
        };
        let mut part = Pen::new();
        part.solid(&tray.ring(2), 4.0, 7.0, Look::new(Ink::Mid).bevel(0.4));
        part.on_top(
            &[[-14.0, y + 7.0], [16.0, y + 7.0]],
            11.0,
            false,
            Stroke::new(Ink::Faint),
        );
        part.dots(
            &[
                [-8.0, y + 7.0, 11.0],
                [2.0, y + 7.0, 11.0],
                [12.0, y + 7.0, 11.0],
            ],
            0.9,
            Ink::Hi,
            Detail::Near,
        );
        scene.put([1.0, y + 7.0, 7.0], part, vec![]);
    }

    scene.finish()
}
