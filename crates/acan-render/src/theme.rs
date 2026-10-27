//! The light theme, from the kit's iso.css (MIT). Colours live here and nowhere in a figure.

use crate::draw_list::{Fill, Ink};

/// Palette slots on the GPU: fills, then inks, then four agent slots.
pub const PALETTE_LEN: usize = 22;
/// Tint slots. Slot 0 is the identity; instances pick a slot to tint a machine.
pub const TINT_LEN: usize = 8;

const fn rgb(hex: u32, alpha: f32) -> [f32; 4] {
    [
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
        alpha,
    ]
}

/// Page background, sRGB.
pub const PAGE: [f32; 4] = rgb(0xffffff, 1.0);

/// sRGB, straight alpha. Agent slots are placeholders until agents get a palette.
const PALETTE: [[f32; 4]; PALETTE_LEN] = [
    rgb(0xffffff, 1.0),  // 0 Face
    rgb(0xffffff, 1.0),  // 1 Top
    rgb(0xdcdcd9, 1.0),  // 2 Shade0
    rgb(0xe5e5e2, 1.0),  // 3 Shade1
    rgb(0xededea, 1.0),  // 4 Shade2
    rgb(0xf4f4f1, 1.0),  // 5 Shade3
    rgb(0xffffff, 1.0),  // 6 LitTop
    rgb(0xeeeeeb, 1.0),  // 7 LitShade
    rgb(0x8a8a80, 0.32), // 8 Halo
    rgb(0xfbfbfa, 1.0),  // 9 Paper
    rgb(0x26262a, 1.0),  // 10 Hi
    rgb(0x8a8a8f, 1.0),  // 11 Mid
    rgb(0xb4b4b8, 1.0),  // 12 Lo
    rgb(0xd4d4d6, 1.0),  // 13 Faint
    rgb(0x000000, 1.0),  // 14 Lit
    rgb(0xe5484d, 1.0),  // 15 Red
    rgb(0x30a46c, 1.0),  // 16 Green
    rgb(0x3e63dd, 1.0),  // 17 Blue
    rgb(0x8e4ec6, 1.0),  // 18 Agent 0 (placeholder)
    rgb(0xf76b15, 1.0),  // 19 Agent 1 (placeholder)
    rgb(0x12a594, 1.0),  // 20 Agent 2 (placeholder)
    rgb(0xab6400, 1.0),  // 21 Agent 3 (placeholder)
];

pub fn fill_slot(tone: Fill) -> u32 {
    match tone {
        Fill::Face => 0,
        Fill::Top => 1,
        Fill::Shade0 => 2,
        Fill::Shade1 => 3,
        Fill::Shade2 => 4,
        Fill::Shade3 => 5,
        Fill::LitTop => 6,
        Fill::LitShade => 7,
        Fill::Halo => 8,
        Fill::Paper => 9,
    }
}

pub fn ink_slot(tone: Ink) -> u32 {
    match tone {
        Ink::Hi => 10,
        Ink::Mid => 11,
        Ink::Lo => 12,
        Ink::Faint => 13,
        Ink::Lit => 14,
        Ink::Red => 15,
        Ink::Green => 16,
        Ink::Blue => 17,
        Ink::Agent(slot) => 18 + u32::from(slot % 4),
    }
}

/// The palette as the surface wants it: linear when the surface is an sRGB format.
pub fn palette(srgb_surface: bool) -> [[f32; 4]; PALETTE_LEN] {
    PALETTE.map(|c| if srgb_surface { to_linear(c) } else { c })
}

pub fn tints() -> [[f32; 4]; TINT_LEN] {
    [[1.0; 4]; TINT_LEN]
}

pub fn page(srgb_surface: bool) -> [f32; 4] {
    if srgb_surface { to_linear(PAGE) } else { PAGE }
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
    fn slots_are_distinct_and_in_range() {
        let fills = [
            Fill::Face,
            Fill::Top,
            Fill::Shade0,
            Fill::Shade1,
            Fill::Shade2,
            Fill::Shade3,
            Fill::LitTop,
            Fill::LitShade,
            Fill::Halo,
            Fill::Paper,
        ];
        let inks = [
            Ink::Hi,
            Ink::Mid,
            Ink::Lo,
            Ink::Faint,
            Ink::Lit,
            Ink::Red,
            Ink::Green,
            Ink::Blue,
        ];
        let mut slots: Vec<u32> = fills.iter().map(|f| fill_slot(*f)).collect();
        slots.extend(inks.iter().map(|i| ink_slot(*i)));
        slots.extend((0..4).map(|n| ink_slot(Ink::Agent(n))));
        let mut sorted = slots.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), slots.len());
        assert!(slots.iter().all(|s| (*s as usize) < PALETTE_LEN));
    }

    #[test]
    fn halo_is_the_kit_grey_at_a_third_opacity() {
        let halo = PALETTE[fill_slot(Fill::Halo) as usize];
        assert!((halo[3] - 0.32).abs() < 1e-6);
        assert!((halo[0] - 0x8a as f32 / 255.0).abs() < 1e-6);
    }
}
