use crate::model::{Aabb, Detail, Line3, Mesh};

/// One machine on the plan: a model kind placed at an offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    /// Where the model's origin sits. The model's plan centre is on the ground at `z = 0`.
    pub offset: [f32; 3],
    /// Index into the models given to `Renderer::set_models`.
    pub kind: u16,
    /// Index into the theme's tint slots. Slot 0 is the identity.
    pub tint: u8,
}

/// What the renderer draws, set once per plant change.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    /// Static geometry: belts and anything else that is not a model instance.
    pub world: Mesh,
    /// Static lines in world space, such as the edges of a belt.
    pub lines: Vec<Line3>,
    pub instances: Vec<Instance>,
    /// Box around everything in the scene, used to size the depth range.
    pub bounds: Aabb,
}

/// Level of detail, picked from zoom relative to the fit zoom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Lod {
    Far = 0,
    Near = 1,
    Close = 2,
}

impl Lod {
    /// Zoom divided by fit zoom at which `Near` detail appears.
    pub const NEAR_RATIO: f32 = 1.0;
    /// Zoom divided by fit zoom at which `Close` detail appears.
    pub const CLOSE_RATIO: f32 = 2.5;

    pub fn for_ratio(ratio: f32) -> Self {
        if ratio >= Self::CLOSE_RATIO {
            Self::Close
        } else if ratio >= Self::NEAR_RATIO {
            Self::Near
        } else {
            Self::Far
        }
    }

    /// Whether detail authored at `detail` is drawn at this level.
    pub fn shows(self, detail: Detail) -> bool {
        detail as u8 <= self as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_switch_at_the_documented_ratios() {
        assert_eq!(Lod::for_ratio(0.99), Lod::Far);
        assert_eq!(Lod::for_ratio(1.0), Lod::Near);
        assert_eq!(Lod::for_ratio(2.49), Lod::Near);
        assert_eq!(Lod::for_ratio(2.5), Lod::Close);
    }

    #[test]
    fn each_level_shows_its_own_detail_and_the_ones_below() {
        assert!(Lod::Far.shows(Detail::Always));
        assert!(!Lod::Far.shows(Detail::Near));
        assert!(Lod::Near.shows(Detail::Near));
        assert!(!Lod::Near.shows(Detail::Close));
        assert!(Lod::Close.shows(Detail::Close));
    }
}
