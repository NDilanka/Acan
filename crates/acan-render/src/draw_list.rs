//! The draw list a figure hands to the renderer. Field for field this is `acan_iso::Figure`
//! (crates/acan-iso/src/figure.rs), so a real figure replaces a stand-in without a conversion.

/// One model, already projected to the 2D iso plane at scale 1 (plan units, y down).
/// The origin is the projection of the model's plan origin at z = 0. Items are in painter
/// order, back to front.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawList {
    pub items: Vec<Item>,
    /// min x, min y, max x, max y.
    pub bounds: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A simple polygon (may be concave), filled flat.
    Fill { ring: Vec<[f32; 2]>, tone: Fill },
    /// A polyline or closed outline. `width` is in plan units at scale 1.
    Line {
        points: Vec<[f32; 2]>,
        closed: bool,
        tone: Ink,
        width: f32,
        detail: Detail,
    },
    /// Round dots such as screws and lamps. `radius` in plan units.
    Dots {
        points: Vec<[f32; 2]>,
        radius: f32,
        tone: Ink,
        detail: Detail,
    },
}

/// The kit's surface tones. Colours live in the renderer's theme, never in the figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    Face,
    Top,
    Shade0,
    Shade1,
    Shade2,
    Shade3,
    LitTop,
    LitShade,
    Halo,
    Paper,
}

/// The kit's ink tones. `Agent(slot)` is the only saturated ink, reserved for agents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Hi,
    Mid,
    Lo,
    Faint,
    Lit,
    Red,
    Green,
    Blue,
    Agent(u8),
}

/// Level of detail. `Near` and `Close` items drop out when zoomed out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Detail {
    Always = 0,
    Near = 1,
    Close = 2,
}
