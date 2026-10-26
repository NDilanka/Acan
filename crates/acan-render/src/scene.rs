use bytemuck::{Pod, Zeroable};

/// A machine footprint. Uploaded as-is as one GPU instance.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Rect {
    pub min: [f32; 2],
    pub size: [f32; 2],
    /// sRGB, straight alpha.
    pub color: [f32; 4],
}

impl Rect {
    pub fn center(&self) -> [f32; 2] {
        [
            self.min[0] + self.size[0] * 0.5,
            self.min[1] + self.size[1] * 0.5,
        ]
    }
}

/// A belt: a polyline with a constant width in world units.
#[derive(Clone, Debug, PartialEq)]
pub struct Polyline {
    pub points: Vec<[f32; 2]>,
    pub width: f32,
    /// sRGB, straight alpha.
    pub color: [f32; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub rects: Vec<Rect>,
    pub belts: Vec<Polyline>,
}

impl Scene {
    /// World-space bounding box of every rect and belt point, if the scene is not empty.
    pub fn bounds(&self) -> Option<([f32; 2], [f32; 2])> {
        let corners = self
            .rects
            .iter()
            .flat_map(|r| [r.min, [r.min[0] + r.size[0], r.min[1] + r.size[1]]])
            .chain(self.belts.iter().flat_map(|b| b.points.iter().copied()));
        corners.fold(None, |acc, p| {
            let (min, max) = acc.unwrap_or((p, p));
            Some((
                [min[0].min(p[0]), min[1].min(p[1])],
                [max[0].max(p[0]), max[1].max(p[1])],
            ))
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct BeltVertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
}

/// Triangulates a polyline into a triangle list, one quad per segment.
/// Each quad is extended by half the width at both ends (square caps), which
/// closes the corner exactly for the right-angle turns belts use.
pub fn tessellate_polyline(line: &Polyline, out: &mut Vec<BeltVertex>) {
    let half = line.width * 0.5;
    for seg in line.points.windows(2) {
        let (a, b) = (seg[0], seg[1]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len <= f32::EPSILON {
            continue;
        }
        let (ux, uy) = (dx / len * half, dy / len * half);
        let (nx, ny) = (-uy, ux);
        let a = [a[0] - ux, a[1] - uy];
        let b = [b[0] + ux, b[1] + uy];
        let corners = [
            [a[0] + nx, a[1] + ny],
            [a[0] - nx, a[1] - ny],
            [b[0] - nx, b[1] - ny],
            [b[0] + nx, b[1] + ny],
        ];
        out.extend([0, 1, 2, 0, 2, 3].map(|i| BeltVertex {
            pos: corners[i],
            color: line.color,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_segment_becomes_capped_quad() {
        let line = Polyline {
            points: vec![[0.0, 0.0], [10.0, 0.0]],
            width: 2.0,
            color: [1.0; 4],
        };
        let mut v = Vec::new();
        tessellate_polyline(&line, &mut v);
        assert_eq!(v.len(), 6);
        let xs: Vec<f32> = v.iter().map(|v| v.pos[0]).collect();
        let ys: Vec<f32> = v.iter().map(|v| v.pos[1]).collect();
        assert_eq!(xs.iter().cloned().fold(f32::MAX, f32::min), -1.0);
        assert_eq!(xs.iter().cloned().fold(f32::MIN, f32::max), 11.0);
        assert_eq!(ys.iter().cloned().fold(f32::MAX, f32::min), -1.0);
        assert_eq!(ys.iter().cloned().fold(f32::MIN, f32::max), 1.0);
    }

    #[test]
    fn degenerate_segments_are_skipped() {
        let line = Polyline {
            points: vec![[0.0, 0.0], [0.0, 0.0], [0.0, 5.0], [4.0, 5.0]],
            width: 1.0,
            color: [1.0; 4],
        };
        let mut v = Vec::new();
        tessellate_polyline(&line, &mut v);
        assert_eq!(v.len(), 12);
    }

    #[test]
    fn bounds_cover_rects_and_belts() {
        let scene = Scene {
            rects: vec![Rect {
                min: [2.0, 3.0],
                size: [4.0, 5.0],
                color: [0.0; 4],
            }],
            belts: vec![Polyline {
                points: vec![[-1.0, 0.0], [0.0, 20.0]],
                width: 1.0,
                color: [0.0; 4],
            }],
        };
        assert_eq!(scene.bounds(), Some(([-1.0, 0.0], [6.0, 20.0])));
        assert_eq!(Scene::default().bounds(), None);
    }
}
