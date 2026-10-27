//! The surface a part is drawn on. Parts are written in plan coordinates and projected as they are pushed.

use crate::camera::Camera;
use crate::figure::{Detail, Ink, Item};
use crate::geom::Point;

/// The kit's line weight at scale 1.
const WEIGHT: f32 = 0.6;

/// How a line is drawn: its ink, its weight and the detail level it needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Stroke {
    pub(crate) ink: Ink,
    pub(crate) width: f32,
    pub(crate) detail: Detail,
}

impl Stroke {
    pub(crate) fn new(ink: Ink) -> Self {
        Self {
            ink,
            width: WEIGHT,
            detail: Detail::Always,
        }
    }

    pub(crate) fn near(self) -> Self {
        Self {
            detail: Detail::Near,
            ..self
        }
    }

    pub(crate) fn close(self) -> Self {
        Self {
            detail: Detail::Close,
            ..self
        }
    }

    pub(crate) fn width(self, width: f32) -> Self {
        Self { width, ..self }
    }
}

/// How a solid is drawn: the ink of its outline, the ink of its creases (or none), and its bevel width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Look {
    pub(crate) edge: Ink,
    pub(crate) crease: Option<Ink>,
    pub(crate) bevel: f32,
}

impl Look {
    pub(crate) fn new(edge: Ink) -> Self {
        Self {
            edge,
            crease: Some(Ink::Faint),
            bevel: 0.0,
        }
    }

    pub(crate) fn crease(self, crease: Option<Ink>) -> Self {
        Self { crease, ..self }
    }

    pub(crate) fn bevel(self, bevel: f32) -> Self {
        Self { bevel, ..self }
    }
}

/// The items of one part, in the order they should be drawn.
#[derive(Debug)]
pub(crate) struct Pen {
    pub(crate) cam: Camera,
    pub(crate) items: Vec<Item>,
}

impl Default for Pen {
    fn default() -> Self {
        Self::new()
    }
}

impl Pen {
    pub(crate) fn new() -> Self {
        Self {
            cam: Camera::kit(),
            items: Vec::new(),
        }
    }

    /// A polyline through world-space points.
    pub(crate) fn line(&mut self, points: &[[f32; 3]], closed: bool, stroke: Stroke) {
        let projected = points.iter().map(|&p| self.cam.project(p)).collect();
        self.items.push(line_item(projected, closed, stroke));
    }

    /// A polyline through plan points, lying on the plane at height `z`.
    pub(crate) fn on_top(&mut self, points: &[Point], z: f32, closed: bool, stroke: Stroke) {
        let lifted: Vec<[f32; 3]> = points.iter().map(|&[x, y]| [x, y, z]).collect();
        self.line(&lifted, closed, stroke);
    }

    /// Round dots at world-space points.
    pub(crate) fn dots(&mut self, points: &[[f32; 3]], radius: f32, ink: Ink, detail: Detail) {
        let projected = points.iter().map(|&p| self.cam.project(p)).collect();
        self.items.push(Item::Dots {
            points: projected,
            radius,
            tone: ink,
            detail,
        });
    }
}

/// A line item from points that are already projected.
pub(crate) fn line_item(points: Vec<Point>, closed: bool, stroke: Stroke) -> Item {
    Item::Line {
        points,
        closed,
        tone: stroke.ink,
        width: stroke.width,
        detail: stroke.detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_defaults_to_the_kit_weight_and_always() {
        let stroke = Stroke::new(Ink::Lo);
        assert_eq!(stroke.width, 0.6);
        assert_eq!(stroke.detail, Detail::Always);
        assert_eq!(stroke.near().detail, Detail::Near);
        assert_eq!(stroke.close().detail, Detail::Close);
    }

    #[test]
    fn lines_are_projected_as_they_are_pushed() {
        let mut pen = Pen::new();
        pen.line(&[[1.0, 0.0, 0.0]], false, Stroke::new(Ink::Mid));
        pen.on_top(&[[0.0, 0.0]], 1.0, false, Stroke::new(Ink::Mid));
        assert_eq!(pen.items.len(), 2);
        let Item::Line { points, .. } = &pen.items[1] else {
            panic!("expected a line")
        };
        assert!((points[0][1] + 1.095_445).abs() < 1e-4);
    }
}
