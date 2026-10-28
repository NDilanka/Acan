//! The light theme: a white page, neutral greys and fine grey line art. Colours live here and
//! nowhere in a model. The shader reads the slot numbers and the tuning constants from the
//! WGSL that `wgsl_consts` writes, so the two never disagree.

use crate::model::{Ink, Surface};

/// Palette slots on the GPU.
pub const PALETTE_LEN: usize = 22;
/// Tint slots. Slot 0 is the identity; instances pick a slot to tint a machine.
pub const TINT_LEN: usize = 8;

pub const SLOT_PAGE: u32 = 0;
pub const SLOT_LIT: u32 = 1;
pub const SLOT_GLASS: u32 = 2;
/// The soft contact shadow at the foot of a machine.
pub const SLOT_SHADOW: u32 = 3;
pub const SLOT_GRID: u32 = 4;
pub const SLOT_HI: u32 = 5;
pub const SLOT_MID: u32 = 6;
pub const SLOT_LO: u32 = 7;
pub const SLOT_FAINT: u32 = 8;
pub const SLOT_DOT: u32 = 9;
pub const SLOT_RED: u32 = 10;
pub const SLOT_GREEN: u32 = 11;
pub const SLOT_BLUE: u32 = 12;
/// Four agent slots. Placeholders until agents get a palette.
pub const SLOT_AGENT0: u32 = 13;
/// The opaque fill of a belt and the fine line along its sides and ends.
pub const SLOT_BELT: u32 = 17;
pub const SLOT_BELT_EDGE: u32 = 18;
/// The outer outline of every object, drawn in the edge pass.
pub const SLOT_OUTLINE: u32 = 19;
/// The inner feature edges of a model, drawn as thin lines.
pub const SLOT_EDGE: u32 = 20;
/// The outer outline of a machine that is small on screen. The outline eases from this to
/// `SLOT_OUTLINE` over `LOD_FADE_PX`, like the inner lines, so a dense field reads as light
/// blocks and not as dark speckle.
pub const SLOT_OUTLINE_FAR: u32 = 21;

/// Paper shading, as darkness below white. The light is fixed to the camera, from its left,
/// so a face keeps its tone at every azimuth. A top is nearly white, a side facing the light
/// is `PAPER_LIT` darker and a side facing away is `PAPER_DARK` darker. The two sides a box
/// shows at the default view sit at 7% and 14%.
pub const PAPER_TOP: f32 = 0.015;
pub const PAPER_LIT: f32 = 0.056;
pub const PAPER_DARK: f32 = 0.154;

/// Distance fog from the eye. It starts past the plant's farthest corner, and never nearer
/// than `FOG_NEAR` times the pivot's eye distance, so the plant and the ground under it stay
/// clear. It ends `FOG_SPAN` times as far as it starts, which leaves the grid room to
/// converge toward a soft horizon. Fog mixes toward the page colour.
pub const FOG_NEAR: f32 = 1.5;
pub const FOG_SPAN: f32 = 8.0;
/// Lines fade as the view grazes the ground, from `ANGLE_FADE.0` to `ANGLE_FADE.1` of the
/// absolute sine of the ray's elevation. This kills moire near the horizon, as Blender does.
pub const ANGLE_FADE: (f32, f32) = (0.004, 0.05);
/// Grid levels, each `GRID_STEP` times the spacing of the one before, starting at `GRID_BASE`
/// plan units.
pub const GRID_LEVELS: u32 = 4;
pub const GRID_BASE: f32 = 8.0;
pub const GRID_STEP: f32 = 5.0;
/// A level shows while its on-screen spacing is between these device pixels, so levels cross
/// fade as the zoom changes.
pub const GRID_SHOW_PX: (f32, f32) = (6.0, 30.0);
/// Ink of a grid line, as black alpha on the white page. A level is minor while the finer
/// level under it is still hidden and turns major as that level appears. A pixel takes the
/// strongest level, never the sum, so the grid stays under the machine line art.
pub const GRID_MINOR_ALPHA: f32 = 0.05;
pub const GRID_MAJOR_ALPHA: f32 = 0.11;

/// The contact shadow: how far it reaches past the footprint, in plan units, and how far the
/// light pushes it toward the camera's right.
pub const SHADOW_REACH: f32 = 7.0;
pub const SHADOW_SHIFT: f32 = 2.0;

/// The belt's side and end lines are this wide, in device pixels. They fade as the belt's
/// width across it on screen narrows from `BELT_EDGE_FADE_PX.1` to `BELT_EDGE_FADE_PX.0`
/// device pixels, but never below `BELT_EDGE_FLOOR`, so a far belt stays a quiet strip and
/// never vanishes or dots.
pub const BELT_EDGE_PX: f32 = 1.0;
pub const BELT_EDGE_FADE_PX: (f32, f32) = (3.0, 9.0);
pub const BELT_EDGE_FLOOR: f32 = 0.35;

/// Inner lines (feature edges and painted detail) fade out as a machine shrinks on screen.
/// The size is the screen length of `LOD_REFERENCE` plan units at the machine. Below
/// `LOD_FADE_PX.0` device pixels a machine is only its outline.
pub const LOD_REFERENCE: f32 = 96.0;
pub const LOD_FADE_PX: (f32, f32) = (30.0, 60.0);

const fn rgb(hex: u32, alpha: f32) -> [f32; 4] {
    [
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
        alpha,
    ]
}

/// sRGB with straight alpha.
const PALETTE: [[f32; 4]; PALETTE_LEN] = [
    rgb(0xffffff, 1.0),  // page
    rgb(0xffffff, 1.0),  // lit
    rgb(0xf2f2f2, 1.0),  // glass
    rgb(0x000000, 0.08), // contact shadow
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
    rgb(0xe9e9e9, 1.0),  // belt
    rgb(0xb8b8b8, 1.0),  // belt edge
    rgb(0x6b6b6b, 0.9),  // outline
    rgb(0x9a9a9a, 0.6),  // feature edge
    rgb(0x8a8a8a, 0.85), // outline of a far machine
];

/// Palette slot of an ink used by a line.
pub fn ink_slot(ink: Ink) -> u32 {
    match ink {
        Ink::Hi => SLOT_HI,
        Ink::Mid => SLOT_MID,
        Ink::Lo => SLOT_LO,
        Ink::Faint => SLOT_FAINT,
        Ink::Edge => SLOT_EDGE,
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

/// The per-vertex code of a surface: 0 for paper (shaded in the shader), otherwise the palette
/// slot plus one.
pub fn flat_code(surface: Surface) -> u32 {
    match surface {
        Surface::Paper => 0,
        Surface::Lit => SLOT_LIT + 1,
        Surface::Glass => SLOT_GLASS + 1,
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

/// The slots and tuning constants the shader reads, as WGSL constant declarations.
pub fn wgsl_consts() -> String {
    let f = |name: &str, value: f32| format!("const {name}: f32 = {value:?};\n");
    let u = |name: &str, value: u32| format!("const {name}: u32 = {value}u;\n");
    let pair = |name: &str, (lo, hi): (f32, f32)| {
        format!("const {name}: vec2<f32> = vec2<f32>({lo:?}, {hi:?});\n")
    };
    [
        u("SLOT_PAGE", SLOT_PAGE),
        u("SLOT_SHADOW", SLOT_SHADOW),
        u("SLOT_GRID", SLOT_GRID),
        u("SLOT_BELT", SLOT_BELT),
        u("SLOT_BELT_EDGE", SLOT_BELT_EDGE),
        u("SLOT_OUTLINE", SLOT_OUTLINE),
        u("SLOT_OUTLINE_FAR", SLOT_OUTLINE_FAR),
        f("PAPER_TOP", PAPER_TOP),
        f("PAPER_LIT", PAPER_LIT),
        f("PAPER_DARK", PAPER_DARK),
        format!("const GRID_LEVELS: i32 = {};\n", GRID_LEVELS),
        f("GRID_BASE", GRID_BASE),
        f("GRID_STEP", GRID_STEP),
        f("GRID_MINOR_ALPHA", GRID_MINOR_ALPHA),
        f("GRID_MAJOR_ALPHA", GRID_MAJOR_ALPHA),
        pair("GRID_SHOW_PX", GRID_SHOW_PX),
        f("SHADOW_REACH", SHADOW_REACH),
        f("SHADOW_SHIFT", SHADOW_SHIFT),
        f("BELT_EDGE_PX", BELT_EDGE_PX),
        pair("BELT_EDGE_FADE_PX", BELT_EDGE_FADE_PX),
        f("BELT_EDGE_FLOOR", BELT_EDGE_FLOOR),
        f("LOD_REFERENCE", LOD_REFERENCE),
        pair("LOD_FADE_PX", LOD_FADE_PX),
    ]
    .concat()
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

const _: () = assert!(
    SLOT_AGENT0 + 3 < SLOT_BELT,
    "agents do not run into the belt slots"
);
const _: () = assert!(GRID_MINOR_ALPHA < GRID_MAJOR_ALPHA);
const _: () = assert!(PAPER_TOP > 0.0, "a top never matches the page");
const _: () = assert!(PAPER_TOP < PAPER_LIT && PAPER_LIT < PAPER_DARK);
const _: () = assert!(PAPER_DARK < 0.2, "no face turns dark");

#[cfg(test)]
mod tests {
    use super::*;

    /// Ink coverage of a colour on the white page: 0 is the page, 1 is black.
    fn ink(c: [f32; 4]) -> f32 {
        (1.0 - c[0]) * c[3]
    }

    #[test]
    fn every_ink_maps_to_its_own_slot_in_range() {
        let inks = [
            Ink::Hi,
            Ink::Mid,
            Ink::Lo,
            Ink::Faint,
            Ink::Edge,
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
        assert_eq!(ink_slot(Ink::Agent(3)), SLOT_AGENT0 + 3);
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
    fn the_grid_stays_under_the_line_art() {
        let grid = PALETTE[SLOT_GRID as usize];
        assert_eq!(&grid[..3], &[0.0, 0.0, 0.0]);
        let faintest_line = [SLOT_EDGE, SLOT_OUTLINE, SLOT_OUTLINE_FAR, SLOT_BELT_EDGE]
            .map(|slot| ink(PALETTE[slot as usize]))
            .into_iter()
            .fold(f32::INFINITY, f32::min);
        assert!(
            GRID_MAJOR_ALPHA < faintest_line,
            "major grid ink {GRID_MAJOR_ALPHA} must stay under every line, {faintest_line}"
        );
    }

    #[test]
    fn the_outline_is_darker_than_the_inner_edges() {
        let outline = PALETTE[SLOT_OUTLINE as usize];
        let edge = PALETTE[SLOT_EDGE as usize];
        assert!(ink(outline) > ink(edge));
        assert!(ink(outline) < 0.7, "the outline is grey, not black");
        let far = PALETTE[SLOT_OUTLINE_FAR as usize];
        assert!(ink(far) < ink(outline), "a far outline is lighter");
        assert!(ink(far) > ink(edge));
    }

    #[test]
    fn a_belt_is_a_step_darker_than_a_plate_top() {
        let belt = ink(PALETTE[SLOT_BELT as usize]);
        assert!(
            belt > PAPER_TOP && belt < PAPER_LIT * 2.0,
            "belt ink {belt}"
        );
        assert!(ink(PALETTE[SLOT_BELT_EDGE as usize]) > belt);
    }

    #[test]
    fn the_shader_constants_name_every_slot_it_reads() {
        let consts = wgsl_consts();
        for name in [
            "SLOT_PAGE",
            "SLOT_SHADOW",
            "SLOT_GRID",
            "SLOT_BELT",
            "SLOT_BELT_EDGE",
            "SLOT_OUTLINE",
        ] {
            assert!(consts.contains(&format!("const {name}: u32")), "{name}");
        }
        assert!(consts.contains(&format!("const SLOT_OUTLINE: u32 = {SLOT_OUTLINE}u;")));
    }
}
