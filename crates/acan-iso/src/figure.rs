//! The draw list a machine model produces. The renderer draws it, and its theme supplies the colours.

/// One model projected to the iso plane at scale 1, in plan units with y down.
///
/// The origin is the projection of the model's plan origin at z = 0, which is the centre of its footprint.
/// Items are in painter order, back to front.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    /// Items in painter order.
    pub items: Vec<Item>,
    /// Min x, min y, max x, max y. Strokes grow by half their width and dots by their radius.
    pub bounds: [f32; 4],
}

impl Figure {
    /// Measures the bounds of `items`, which must already be in painter order.
    pub(crate) fn from_items(items: Vec<Item>) -> Self {
        let mut bounds = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        for item in &items {
            match item {
                Item::Fill { ring, .. } => {
                    for &point in ring {
                        grow(&mut bounds, point, 0.0);
                    }
                }
                Item::Line { points, width, .. } => {
                    for &point in points {
                        grow(&mut bounds, point, *width / 2.0);
                    }
                }
                Item::Dots { points, radius, .. } => {
                    for &point in points {
                        grow(&mut bounds, point, *radius);
                    }
                }
            }
        }
        if items.is_empty() {
            bounds = [0.0; 4];
        }
        Self { items, bounds }
    }
}

fn grow(bounds: &mut [f32; 4], [x, y]: [f32; 2], by: f32) {
    bounds[0] = bounds[0].min(x - by);
    bounds[1] = bounds[1].min(y - by);
    bounds[2] = bounds[2].max(x + by);
    bounds[3] = bounds[3].max(y + by);
}

/// One drawn thing: a filled polygon, a polyline or outline, or a set of round dots.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A simple polygon, filled flat.
    Fill { ring: Vec<[f32; 2]>, tone: Fill },
    /// A polyline, or a closed outline when `closed` is set. `width` is in plan units at scale 1.
    Line {
        points: Vec<[f32; 2]>,
        closed: bool,
        tone: Ink,
        width: f32,
        detail: Detail,
    },
    /// Round dots such as screws and lamps. `radius` is in plan units.
    Dots {
        points: Vec<[f32; 2]>,
        radius: f32,
        tone: Ink,
        detail: Detail,
    },
}

/// Surface tones from the kit. The renderer maps each one to a colour in its theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// The paper a figure sits on. No model draws with it.
    Paper,
    /// The silhouette of a solid, under its sides and top.
    Face,
    /// The top face of a solid.
    Top,
    /// Side faces, darkest (0) to lightest (3).
    Shade0,
    Shade1,
    Shade2,
    Shade3,
    /// The top face of a lit solid.
    LitTop,
    /// The side faces of a lit solid.
    LitShade,
    /// The soft shadow under a body. The renderer blurs it.
    Halo,
}

/// Ink tones from the kit. `Agent(slot)` is the only saturated ink, reserved for agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// The subject: the parts the reader is meant to read, and the pointers on them.
    Hi,
    /// Structure: the bodies, rings and major ticks around the subject.
    Mid,
    /// The base and the finest marks: feet, minor ticks, knurls.
    Lo,
    /// Grooves and creases.
    Faint,
    /// The one accent per figure: the part the reader should watch.
    Lit,
    /// A status colour.
    Red,
    /// A status colour.
    Green,
    /// A status colour.
    Blue,
    /// An agent's slot. The models do not use it.
    Agent(u8),
}

/// The level of detail an item needs. Items at a higher level drop out when the view is zoomed out.
///
/// `Always` draws the silhouette and the main structure. `Near` draws fine parts that show at moderate zoom,
/// such as major ticks, seams, screws and bevels. `Close` draws the finest parts, such as minor ticks, knurls
/// and dot grids, which show only when zoomed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Detail {
    Always,
    Near,
    Close,
}
