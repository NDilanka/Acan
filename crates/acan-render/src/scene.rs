use crate::draw_list::{Detail, DrawList};

/// One machine on the plan: a model kind placed at a projected offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    /// Projected position of the model's plan origin (see `projection::project`).
    pub offset: [f32; 2],
    /// Index into the models given to `Renderer::set_models`.
    pub kind: u16,
    /// Index into the theme's tint slots.
    pub tint: u8,
}

/// What the renderer draws. `instances` must be in painter order, back to front.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    /// Static geometry on the ground: floor, belts and anything else flat.
    pub world: DrawList,
    /// A soft halo that fades out beyond this plan rectangle `(min, max)`, if set.
    pub floor_halo: Option<([f32; 2], [f32; 2])>,
    pub instances: Vec<Instance>,
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
    /// Zoom divided by fit zoom at which `Near` items appear.
    pub const NEAR_RATIO: f32 = 1.0;
    /// Zoom divided by fit zoom at which `Close` items appear.
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
