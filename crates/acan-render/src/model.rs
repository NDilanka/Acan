//! The 3D model a machine kind is drawn from. The shape follows the shared contract
//! (`ui-figure-contract-v2`): models are view-independent meshes, and the renderer does
//! projection, depth, shading and outlines.

/// A model in plan units: right-handed, z up, origin at the plan centre on the ground.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    /// One mesh per level of detail, indexed by `Detail as usize`. A view draws `Always`, plus
    /// `Near` as zoom allows, plus `Close` as zoom allows.
    pub meshes: [Mesh; 3],
    /// Painted detail on surfaces: ticks, seams, needles, rulers. Silhouettes and creases are
    /// not authored; the renderer draws them.
    pub lines: Vec<Line3>,
    /// Screws, lamps, LEDs: discs lying on a surface.
    pub dots: Vec<Dot3>,
    pub bounds: Aabb,
}

/// Triangles with per-vertex normals and surfaces.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<[f32; 3]>,
    /// Smooth normals on curved surfaces, split normals at hard edges.
    pub normals: Vec<[f32; 3]>,
    /// How each vertex is shaded.
    pub surfaces: Vec<Surface>,
    /// Triangle list, counter-clockwise seen from outside.
    pub indices: Vec<u32>,
}

/// A polyline on a surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Line3 {
    pub points: Vec<[f32; 3]>,
    pub closed: bool,
    pub tone: Ink,
    /// Stroke width in device pixels at every zoom. The renderer keeps it at least 0.75 px.
    pub width: f32,
    pub detail: Detail,
}

/// A disc lying on a surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Dot3 {
    pub center: [f32; 3],
    pub normal: [f32; 3],
    /// Radius in plan units.
    pub radius: f32,
    pub tone: Ink,
    pub detail: Detail,
}

/// How a surface is shaded. Paper gets the kit's four-tone light shading from its normal; the
/// others are flat fills.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Paper,
    Lit,
    Glass,
    /// A flat fill lying on the ground, such as a belt. The edge pass treats it as ground.
    Floor,
    Ink(Ink),
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

/// Level of detail. Items at `Near` and `Close` drop out when zoomed out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Detail {
    #[default]
    Always = 0,
    Near = 1,
    Close = 2,
}

/// Axis-aligned box: `min` and `max` corners.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    /// The empty box, which grows to fit the first point added to it.
    pub const EMPTY: Self = Self {
        min: [f32::INFINITY; 3],
        max: [f32::NEG_INFINITY; 3],
    };

    /// Grows the box to contain `p`.
    pub fn include(&mut self, p: [f32; 3]) {
        for (axis, &v) in p.iter().enumerate() {
            self.min[axis] = self.min[axis].min(v);
            self.max[axis] = self.max[axis].max(v);
        }
    }

    /// Centre of the box.
    pub fn centre(&self) -> [f32; 3] {
        std::array::from_fn(|axis| (self.min[axis] + self.max[axis]) * 0.5)
    }

    /// The eight corners.
    pub fn corners(&self) -> [[f32; 3]; 8] {
        std::array::from_fn(|i| {
            std::array::from_fn(|axis| {
                if i >> axis & 1 == 1 {
                    self.max[axis]
                } else {
                    self.min[axis]
                }
            })
        })
    }
}
