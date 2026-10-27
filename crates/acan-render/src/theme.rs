//! The light theme, from the kit's iso.css (MIT) and the shared figure contract. Colours live
//! here and nowhere in a model.

use crate::model::{Ink, Surface};

/// Palette slots on the GPU.
pub const PALETTE_LEN: usize = 22;
/// Tint slots. Slot 0 is the identity; instances pick a slot to tint a machine.
pub const TINT_LEN: usize = 8;

pub const SLOT_PAGE: u32 = 0;
pub const SLOT_TOP: u32 = 1;
/// Side faces, darkest first. Paper picks one of these four from its normal.
pub const SLOT_SHADE0: u32 = 2;
pub const SLOT_SHADE1: u32 = 3;
pub const SLOT_SHADE2: u32 = 4;
pub const SLOT_SHADE3: u32 = 5;
pub const SLOT_LIT: u32 = 6;
pub const SLOT_GLASS: u32 = 7;
pub const SLOT_HALO: u32 = 8;
pub const SLOT_GRID: u32 = 9;
pub const SLOT_HI: u32 = 10;
pub const SLOT_MID: u32 = 11;
pub const SLOT_LO: u32 = 12;
pub const SLOT_FAINT: u32 = 13;
pub const SLOT_DOT: u32 = 14;
pub const SLOT_RED: u32 = 15;
pub const SLOT_GREEN: u32 = 16;
pub const SLOT_BLUE: u32 = 17;
/// Four agent slots. Placeholders until agents get a palette.
pub const SLOT_AGENT0: u32 = 18;

/// Set on the code of a surface that lies on the ground, such as a belt. The edge pass treats it
/// as ground, so its edges come from the ink lines drawn on it. Matches `FLOOR_FLAG` in the shader.
pub const FLOOR_FLAG: u32 = 1 << 16;

/// The grid fades by ground distance from the camera target, in half view heights (H) so the
/// look is the same at every zoom. It is full at the target and gone past `GRID_FADE_END`.
pub const GRID_FADE_START: f32 = 1.5;
pub const GRID_FADE_END: f32 = 4.0;
/// The grid also thins as the view grazes the ground: the fade runs over sin(elevation) in this
/// range.
pub const GRID_GRAZING: (f32, f32) = (0.05, 0.35);
/// Grid levels, each `GRID_STEP` times the spacing of the one before, starting at `GRID_BASE`
/// plan units.
pub const GRID_LEVELS: u32 = 4;
pub const GRID_BASE: f32 = 8.0;
pub const GRID_STEP: f32 = 5.0;
/// A level shows while its on-screen spacing is between these device pixels, so levels cross
/// fade as the zoom changes.
pub const GRID_SHOW_PX: (f32, f32) = (6.0, 30.0);
/// The alpha one level adds on a line, and the cap on the sum over levels.
pub const GRID_LEVEL_ALPHA: f32 = 0.07;
pub const GRID_MAX_ALPHA: f32 = 0.14;

const fn rgb(hex: u32, alpha: f32) -> [f32; 4] {
    [
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
        alpha,
    ]
}

/// sRGB with straight alpha. Glass has no colour in the contract, so it takes shade-3.
const PALETTE: [[f32; 4]; PALETTE_LEN] = [
    rgb(0xffffff, 1.0),  // page
    rgb(0xffffff, 1.0),  // top
    rgb(0xdcdcd9, 1.0),  // shade-0
    rgb(0xe5e5e2, 1.0),  // shade-1
    rgb(0xededea, 1.0),  // shade-2
    rgb(0xf4f4f1, 1.0),  // shade-3
    rgb(0xffffff, 1.0),  // lit
    rgb(0xf4f4f1, 1.0),  // glass
    rgb(0x8a8a80, 0.32), // halo
    rgb(0x000000, 1.0),  // grid, at its alpha
    rgb(0x26262a, 1.0),  // hi
    rgb(0x8a8a8f, 1.0),  // mid
    rgb(0xb4b4b8, 1.0),  // lo
    rgb(0xd4d4d6, 1.0),  // faint
    rgb(0x3a3a3e, 1.0),  // dot
    rgb(0xe5484d, 1.0),  // red
    rgb(0x30a46c, 1.0),  // green
    rgb(0x3e63dd, 1.0),  // blue
    rgb(0x8e4ec6, 1.0),  // agent 0 (placeholder)
    rgb(0xf76b15, 1.0),  // agent 1 (placeholder)
    rgb(0x12a594, 1.0),  // agent 2 (placeholder)
    rgb(0xab6400, 1.0),  // agent 3 (placeholder)
];

/// Palette slot of an ink used by a line.
pub fn ink_slot(ink: Ink) -> u32 {
    match ink {
        Ink::Hi => SLOT_HI,
        Ink::Mid => SLOT_MID,
        Ink::Lo => SLOT_LO,
        Ink::Faint => SLOT_FAINT,
        Ink::Lit => SLOT_LIT,
        Ink::Red => SLOT_RED,
        Ink::Green => SLOT_GREEN,
        Ink::Blue => SLOT_BLUE,
        Ink::Agent(slot) => SLOT_AGENT0 + u32::from(slot % 4),
    }
}

/// Palette slot of a dot. A hi dot is the kit's dot ink, a little lighter than a hi line.
pub fn dot_slot(ink: Ink) -> u32 {
    match ink {
        Ink::Hi => SLOT_DOT,
        other => ink_slot(other),
    }
}

/// The per-vertex code of a flat surface: 0 for paper (shaded in the shader), otherwise the
/// palette slot plus one, with `FLOOR_FLAG` on a surface that lies on the ground.
pub fn flat_code(surface: Surface) -> u32 {
    match surface {
        Surface::Paper => 0,
        Surface::Lit => SLOT_LIT + 1,
        Surface::Glass => SLOT_GLASS + 1,
        Surface::Floor => (SLOT_SHADE1 + 1) | FLOOR_FLAG,
        Surface::Ink(ink) => ink_slot(ink) + 1,
    }
}

/// The palette as the surface wants it: linear when the surface is an sRGB format.
pub fn palette(srgb_surface: bool) -> [[f32; 4]; PALETTE_LEN] {
    PALETTE.map(|c| if srgb_surface { to_linear(c) } else { c })
}

pub fn tints() -> [[f32; 4]; TINT_LEN] {
    [[1.0; 4]; TINT_LEN]
}

/// The page colour for the clear, linear when the surface is an sRGB format.
pub fn page(srgb_surface: bool) -> [f32; 4] {
    palette(srgb_surface)[SLOT_PAGE as usize]
}

fn to_linear(c: [f32; 4]) -> [f32; 4] {
    let f = |x: f32| {
        if x <= 0.040_45 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ink_maps_to_its_own_slot_in_range() {
        let inks = [
            Ink::Hi,
            Ink::Mid,
            Ink::Lo,
            Ink::Faint,
            Ink::Lit,
            Ink::Red,
            Ink::Green,
            Ink::Blue,
            Ink::Agent(0),
            Ink::Agent(3),
        ];
        let slots: Vec<u32> = inks.iter().map(|&ink| ink_slot(ink)).collect();
        assert!(slots.iter().all(|&s| (s as usize) < PALETTE_LEN));
        assert_eq!(ink_slot(Ink::Agent(0)), ink_slot(Ink::Agent(4)));
        assert_eq!(dot_slot(Ink::Hi), SLOT_DOT);
        assert_eq!(dot_slot(Ink::Mid), SLOT_MID);
    }

    #[test]
    fn flat_codes_leave_zero_for_paper() {
        assert_eq!(flat_code(Surface::Paper), 0);
        assert_eq!(flat_code(Surface::Lit), SLOT_LIT + 1);
        assert_eq!(flat_code(Surface::Ink(Ink::Hi)), SLOT_HI + 1);
    }

    #[test]
    fn floor_surfaces_carry_the_flag_and_the_shade_one_slot() {
        let code = flat_code(Surface::Floor);
        assert_ne!(code & FLOOR_FLAG, 0);
        assert_eq!((code & !FLOOR_FLAG) - 1, SLOT_SHADE1);
        assert_eq!(flat_code(Surface::Lit) & FLOOR_FLAG, 0);
    }

    #[test]
    fn palette_ends_at_the_last_agent_slot_and_the_grid_is_black() {
        assert_eq!(ink_slot(Ink::Agent(3)) as usize, PALETTE_LEN - 1);
        let grid = PALETTE[SLOT_GRID as usize];
        assert_eq!(&grid[..3], &[0.0, 0.0, 0.0]);
        assert!(grid[3] > 0.0);
    }

    #[test]
    fn halo_is_the_kit_grey_at_a_third_opacity() {
        let halo = PALETTE[SLOT_HALO as usize];
        assert!((halo[3] - 0.32).abs() < 1e-6);
        assert!((halo[0] - 0x8a as f32 / 255.0).abs() < 1e-6);
    }
}
