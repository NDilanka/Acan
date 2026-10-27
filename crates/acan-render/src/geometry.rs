//! CPU geometry. Each draw list is turned into index ranges once. A zoom change picks
//! other ranges and never rebuilds a vertex. Fills are tessellated with lyon. Strokes and
//! dots are quads that the vertex shader expands to the device width.

use bytemuck::{Pod, Zeroable};
use lyon_tessellation::math::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex as LyonVertex, VertexBuffers,
};

pub use lyon_tessellation::TessellationError;

use crate::draw_list::{Detail, DrawList, Fill, Ink, Item};
use crate::projection::project;
use crate::scene::Lod;
use crate::theme::{fill_slot, ink_slot};

/// Rings in the floor's soft halo. Their alpha falls off quadratically.
const HALO_STEPS: u32 = 8;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct FillVertex {
    pub pos: [f32; 2],
    pub alpha: f32,
    pub tone: u32,
    pub order: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct LineVertex {
    pub a: [f32; 2],
    pub b: [f32; 2],
    /// x in 0..1 along the segment, y in -1..1 across it.
    pub corner: [f32; 2],
    pub width: f32,
    pub tone: u32,
    pub order: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DotVertex {
    pub center: [f32; 2],
    pub corner: [f32; 2],
    pub radius: f32,
    pub tone: u32,
    pub order: f32,
}

/// An index range inside one class's index buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

/// The four primitive classes. Opaque fills, lines and dots write depth. Halo fills
/// are translucent and draw last, depth-tested but not written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Opaque = 0,
    Halo = 1,
    Line = 2,
    Dot = 3,
}

/// Spans per class and per level of detail, for one model.
pub type KindSpans = [[Vec<Span>; 3]; 4];

#[derive(Default)]
pub struct Batch {
    pub fill: Vec<FillVertex>,
    pub fill_opaque: Vec<u32>,
    pub fill_halo: Vec<u32>,
    pub line: Vec<LineVertex>,
    pub line_idx: Vec<u32>,
    pub dot: Vec<DotVertex>,
    pub dot_idx: Vec<u32>,
    /// One entry per pushed draw list, in push order.
    pub kinds: Vec<KindSpans>,
}

impl Batch {
    /// Appends one draw list as the next kind.
    pub fn push_list(&mut self, list: &DrawList) -> Result<(), TessellationError> {
        let mut spans = KindSpans::default();
        let mut tess = FillTessellator::new();
        let n = list.items.len().max(1) as f32;
        for (i, item) in list.items.iter().enumerate() {
            let order = (i as f32 + 0.5) / n;
            match item {
                Item::Fill { ring, tone } => {
                    let class = if *tone == Fill::Halo {
                        Class::Halo
                    } else {
                        Class::Opaque
                    };
                    let (start, end) = self.add_fill(&mut tess, ring, *tone, order, 1.0)?;
                    record(&mut spans, class, start, end, Detail::Always);
                }
                Item::Line {
                    points,
                    closed,
                    tone,
                    width,
                    detail,
                } => {
                    let (start, end) = self.add_line(points, *closed, *tone, *width, order);
                    record(&mut spans, Class::Line, start, end, *detail);
                }
                Item::Dots {
                    points,
                    radius,
                    tone,
                    detail,
                } => {
                    let (start, end) = self.add_dots(points, *radius, *tone, order);
                    record(&mut spans, Class::Dot, start, end, *detail);
                }
            }
        }
        self.kinds.push(spans);
        Ok(())
    }

    /// Appends the floor's soft halo: rings outside `min..max` that fade to nothing.
    /// It draws at the back of the ground, so the floor and machines cover it.
    pub fn push_floor_halo(&mut self, min: [f32; 2], max: [f32; 2], width: f32) {
        let start = self.fill_halo.len() as u32;
        let ring = |e: f32| {
            [
                project(min[0] - e, min[1] - e, 0.0),
                project(max[0] + e, min[1] - e, 0.0),
                project(max[0] + e, max[1] + e, 0.0),
                project(min[0] - e, max[1] + e, 0.0),
            ]
        };
        let alpha = |k: u32| {
            let t = k as f32 / HALO_STEPS as f32;
            (1.0 - t) * (1.0 - t)
        };
        for k in 0..HALO_STEPS {
            let inner = ring(width * k as f32 / HALO_STEPS as f32);
            let outer = ring(width * (k + 1) as f32 / HALO_STEPS as f32);
            for i in 0..4 {
                let j = (i + 1) % 4;
                let base = self.fill.len() as u32;
                for (p, a) in [
                    (inner[i], alpha(k)),
                    (inner[j], alpha(k)),
                    (outer[j], alpha(k + 1)),
                    (outer[i], alpha(k + 1)),
                ] {
                    self.fill.push(FillVertex {
                        pos: p,
                        alpha: a,
                        tone: fill_slot(Fill::Halo),
                        order: 0.0,
                    });
                }
                self.fill_halo
                    .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        let end = self.fill_halo.len() as u32;
        if self.kinds.is_empty() {
            self.kinds.push(KindSpans::default());
        }
        if let Some(spans) = self.kinds.last_mut() {
            for lod in [Lod::Far, Lod::Near, Lod::Close] {
                spans[Class::Halo as usize][lod as usize].push(Span { start, end });
            }
        }
    }

    fn add_fill(
        &mut self,
        tess: &mut FillTessellator,
        ring: &[[f32; 2]],
        tone: Fill,
        order: f32,
        alpha: f32,
    ) -> Result<(u32, u32), TessellationError> {
        let start = self.index_len(tone == Fill::Halo);
        if ring.len() < 3 {
            return Ok((start, start));
        }
        let mut builder = Path::builder();
        builder.begin(point(ring[0][0], ring[0][1]));
        for p in &ring[1..] {
            builder.line_to(point(p[0], p[1]));
        }
        builder.end(true);
        let path = builder.build();
        let mut out: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
        tess.tessellate_path(
            &path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut out, |v: LyonVertex| {
                let p = v.position();
                [p.x, p.y]
            }),
        )?;
        let base = self.fill.len() as u32;
        let tone_slot = fill_slot(tone);
        self.fill.extend(out.vertices.iter().map(|pos| FillVertex {
            pos: *pos,
            alpha,
            tone: tone_slot,
            order,
        }));
        let idx = if tone == Fill::Halo {
            &mut self.fill_halo
        } else {
            &mut self.fill_opaque
        };
        idx.extend(out.indices.iter().map(|i| i + base));
        Ok((start, self.index_len(tone == Fill::Halo)))
    }

    fn add_line(
        &mut self,
        points: &[[f32; 2]],
        closed: bool,
        tone: Ink,
        width: f32,
        order: f32,
    ) -> (u32, u32) {
        let start = self.line_idx.len() as u32;
        let n = points.len();
        if n >= 2 {
            let segments = if closed { n } else { n - 1 };
            let tone_slot = ink_slot(tone);
            for k in 0..segments {
                let a = points[k];
                let b = points[(k + 1) % n];
                if (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) < 1e-12 {
                    continue;
                }
                let base = self.line.len() as u32;
                for corner in [[0.0, -1.0], [0.0, 1.0], [1.0, 1.0], [1.0, -1.0]] {
                    self.line.push(LineVertex {
                        a,
                        b,
                        corner,
                        width,
                        tone: tone_slot,
                        order,
                    });
                }
                self.line_idx
                    .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
        (start, self.line_idx.len() as u32)
    }

    fn add_dots(&mut self, points: &[[f32; 2]], radius: f32, tone: Ink, order: f32) -> (u32, u32) {
        let start = self.dot_idx.len() as u32;
        let tone_slot = ink_slot(tone);
        for &center in points {
            let base = self.dot.len() as u32;
            for corner in [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]] {
                self.dot.push(DotVertex {
                    center,
                    corner,
                    radius,
                    tone: tone_slot,
                    order,
                });
            }
            self.dot_idx
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        (start, self.dot_idx.len() as u32)
    }

    fn index_len(&self, halo: bool) -> u32 {
        if halo {
            self.fill_halo.len() as u32
        } else {
            self.fill_opaque.len() as u32
        }
    }
}

/// Adds the span `start..end` of `class` to every level of detail that shows `detail`.
/// A span that touches the previous one merges into it, so a visible run is one range.
fn record(spans: &mut KindSpans, class: Class, start: u32, end: u32, detail: Detail) {
    if start == end {
        return;
    }
    for lod in [Lod::Far, Lod::Near, Lod::Close] {
        if !lod.shows(detail) {
            continue;
        }
        let list = &mut spans[class as usize][lod as usize];
        match list.last_mut() {
            Some(last) if last.end == start => last.end = end,
            _ => list.push(Span { start, end }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(detail: Detail) -> Item {
        Item::Line {
            points: vec![[0.0, 0.0], [10.0, 0.0]],
            closed: false,
            tone: Ink::Hi,
            width: 0.6,
            detail,
        }
    }

    #[test]
    fn detail_levels_get_their_own_index_ranges() {
        let list = DrawList {
            items: vec![
                line(Detail::Always),
                line(Detail::Near),
                line(Detail::Always),
                line(Detail::Close),
            ],
            bounds: [0.0; 4],
        };
        let mut batch = Batch::default();
        batch.push_list(&list).unwrap();
        let spans = &batch.kinds[0][Class::Line as usize];
        assert_eq!(
            spans[Lod::Far as usize],
            vec![Span { start: 0, end: 6 }, Span { start: 12, end: 18 }]
        );
        assert_eq!(spans[Lod::Near as usize], vec![Span { start: 0, end: 18 }]);
        assert_eq!(spans[Lod::Close as usize], vec![Span { start: 0, end: 24 }]);
    }

    #[test]
    fn closed_outline_has_one_segment_per_edge() {
        let list = DrawList {
            items: vec![Item::Line {
                points: vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
                closed: true,
                tone: Ink::Lo,
                width: 0.6,
                detail: Detail::Always,
            }],
            bounds: [0.0; 4],
        };
        let mut batch = Batch::default();
        batch.push_list(&list).unwrap();
        assert_eq!(batch.line.len(), 16);
        assert_eq!(batch.line_idx.len(), 24);
    }

    #[test]
    fn fills_split_into_opaque_and_halo_classes() {
        let square = vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        let list = DrawList {
            items: vec![
                Item::Fill {
                    ring: square.clone(),
                    tone: Fill::Shade1,
                },
                Item::Fill {
                    ring: square,
                    tone: Fill::Halo,
                },
            ],
            bounds: [0.0; 4],
        };
        let mut batch = Batch::default();
        batch.push_list(&list).unwrap();
        assert!(!batch.fill_opaque.is_empty());
        assert!(!batch.fill_halo.is_empty());
        let spans = &batch.kinds[0];
        assert_eq!(spans[Class::Opaque as usize][0].len(), 1);
        assert_eq!(spans[Class::Halo as usize][0].len(), 1);
    }

    #[test]
    fn a_concave_fill_tessellates_without_error() {
        let l_shape = vec![
            [0.0, 0.0],
            [3.0, 0.0],
            [3.0, 1.0],
            [1.0, 1.0],
            [1.0, 3.0],
            [0.0, 3.0],
        ];
        let list = DrawList {
            items: vec![Item::Fill {
                ring: l_shape,
                tone: Fill::Top,
            }],
            bounds: [0.0; 4],
        };
        let mut batch = Batch::default();
        batch.push_list(&list).unwrap();
        assert_eq!(batch.fill_opaque.len() % 3, 0);
        assert!(batch.fill_opaque.len() >= 12);
    }
}
