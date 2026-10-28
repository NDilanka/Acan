//! Feature edges of a mesh: the creases that a screen-space outline cannot see. The renderer
//! draws them as thin lines in model space, once per model set.
//!
//! The mesh is welded by position, then triangles that lie flat together form patches. Two
//! patches that meet at more than `CREASE_DEGREES` make a crease, and so does an open boundary.
//! A bevel is a short run of patches whose turns each stay under the threshold but add up past
//! it. Such a run draws one line, where half of its turn has happened, so a bevel never draws
//! two lines or three.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::model::{Detail, Ink, Line3, Mesh};

/// Two patches that meet at more than this angle, in degrees, make a crease.
const CREASE_DEGREES: f32 = 50.0;
/// Triangles whose normals differ by less than this, in degrees, are one flat patch.
const FLAT_DEGREES: f32 = 1.0;
/// Positions that round to the same multiple of this, in plan units, are one welded vertex.
const WELD_STEP: f32 = 1e-3;
/// A boundary edge with both ends at this height or lower lies on the ground. The outline pass
/// draws ground contact, so the feature pass skips it.
const GROUND_Z: f32 = 1e-4;
/// Width of a feature line in device pixels. Thinner than an outline, so the inner edges stay
/// faint next to the outer contour.
pub const FEATURE_PX: f32 = 0.75;

/// One line per feature edge of `mesh`, in model space, at one level of detail.
pub fn feature_lines(mesh: &Mesh, detail: Detail) -> Vec<Line3> {
    feature_segments(mesh)
        .into_iter()
        .map(|segment| Line3 {
            points: segment.to_vec(),
            closed: false,
            tone: Ink::Edge,
            width: FEATURE_PX,
            detail,
        })
        .collect()
}

/// The endpoints of each feature edge, in a fixed order. Every welded edge appears at most once.
fn feature_segments(mesh: &Mesh) -> Vec<[[f32; 3]; 2]> {
    let welded = Welded::new(mesh);
    let faces = welded.faces(mesh);
    let mut edge_faces: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (f, face) in faces.iter().enumerate() {
        for k in 0..3 {
            let key = edge_key(face.verts[k], face.verts[(k + 1) % 3]);
            edge_faces.entry(key).or_default().push(f);
        }
    }

    let mut parent: Vec<usize> = (0..faces.len()).collect();
    let flat = FLAT_DEGREES.to_radians().cos();
    for list in edge_faces.values() {
        if let [f, g] = list[..]
            && abs_dot(faces[f].normal, faces[g].normal) >= flat
        {
            let (rf, rg) = (find(&mut parent, f), find(&mut parent, g));
            parent[rf] = rg;
        }
    }
    let patch: Vec<usize> = (0..faces.len()).map(|f| find(&mut parent, f)).collect();

    let mut segments = Vec::new();
    let mut bends: BTreeMap<(usize, usize), Bend> = BTreeMap::new();
    for (&(a, b), list) in &edge_faces {
        let segment = [welded.positions[a as usize], welded.positions[b as usize]];
        match list[..] {
            [_] => {
                if segment[0][2] > GROUND_Z || segment[1][2] > GROUND_Z {
                    segments.push(segment);
                }
            }
            [f, g] => {
                let (p, q) = (patch[f], patch[g]);
                if p == q {
                    continue;
                }
                let turn = angle_degrees(faces[p].normal, faces[q].normal);
                if turn > CREASE_DEGREES {
                    segments.push(segment);
                } else {
                    let ends = (p.min(q), p.max(q));
                    bends
                        .entry(ends)
                        .or_insert_with(|| Bend {
                            ends,
                            turn,
                            segments: Vec::new(),
                        })
                        .segments
                        .push(segment);
                }
            }
            _ => segments.push(segment),
        }
    }
    segments.extend(bevel_lines(bends.into_values().collect()));
    segments
}

/// A smooth join between two patches, with the segments that carry it.
struct Bend {
    ends: (usize, usize),
    turn: f32,
    segments: Vec<[[f32; 3]; 2]>,
}

/// Picks one line per bevel run. A run is a path of smooth joins that starts and ends at a
/// crease. When its turns add up past `CREASE_DEGREES`, the line goes where half the turn has
/// happened. A ring of smooth joins, such as a cylinder's wall, has no ends and draws nothing.
fn bevel_lines(bends: Vec<Bend>) -> Vec<[[f32; 3]; 2]> {
    let mut adjacency: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, bend) in bends.iter().enumerate() {
        adjacency.entry(bend.ends.0).or_default().push(index);
        adjacency.entry(bend.ends.1).or_default().push(index);
    }

    let mut visited: BTreeSet<usize> = BTreeSet::new();
    let mut lines = Vec::new();
    for &start in adjacency.keys() {
        if visited.contains(&start) {
            continue;
        }
        let (nodes, edges) = component(start, &adjacency, &bends);
        visited.extend(nodes.iter().copied());
        let is_path = edges.len() + 1 == nodes.len()
            && nodes.iter().all(|n| adjacency[n].len() <= 2)
            && edges.len() >= 2;
        if !is_path {
            continue;
        }
        let order = path_order(&nodes, &adjacency, &bends);
        let total: f32 = order.iter().map(|&e| bends[e].turn).sum();
        if total <= CREASE_DEGREES {
            continue;
        }
        let mut seen = 0.0;
        for &e in &order {
            seen += bends[e].turn;
            if seen >= 0.5 * total - 1e-3 {
                lines.extend(bends[e].segments.iter().copied());
                break;
            }
        }
    }
    lines
}

/// The nodes and bends reachable from `start` through smooth joins.
fn component(
    start: usize,
    adjacency: &BTreeMap<usize, Vec<usize>>,
    bends: &[Bend],
) -> (Vec<usize>, Vec<usize>) {
    let mut nodes = vec![start];
    let mut edges = Vec::new();
    let mut seen_nodes = BTreeSet::from([start]);
    let mut seen_edges = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(node) = stack.pop() {
        for &e in &adjacency[&node] {
            if seen_edges.insert(e) {
                edges.push(e);
            }
            let other = other_end(bends[e].ends, node);
            if seen_nodes.insert(other) {
                nodes.push(other);
                stack.push(other);
            }
        }
    }
    (nodes, edges)
}

/// The bends of a path, from one end to the other. Only called on a component that is a path.
fn path_order(
    nodes: &[usize],
    adjacency: &BTreeMap<usize, Vec<usize>>,
    bends: &[Bend],
) -> Vec<usize> {
    let Some(&start) = nodes.iter().find(|n| adjacency[n].len() == 1) else {
        return Vec::new();
    };
    let mut order = Vec::new();
    let mut node = start;
    let mut came_by = None;
    while let Some(&next) = adjacency[&node].iter().find(|&&e| Some(e) != came_by) {
        order.push(next);
        node = other_end(bends[next].ends, node);
        came_by = Some(next);
    }
    order
}

fn other_end(ends: (usize, usize), node: usize) -> usize {
    if ends.0 == node { ends.1 } else { ends.0 }
}

fn find(parent: &mut [usize], mut f: usize) -> usize {
    while parent[f] != f {
        parent[f] = parent[parent[f]];
        f = parent[f];
    }
    f
}

#[derive(Clone, Copy)]
struct Face {
    verts: [u32; 3],
    normal: [f32; 3],
}

/// The mesh's positions with coincident vertices merged, so faces that share a corner in space
/// share it in the index, whatever their normals.
struct Welded {
    positions: Vec<[f32; 3]>,
    index: Vec<u32>,
}

impl Welded {
    fn new(mesh: &Mesh) -> Self {
        let mut ids: HashMap<[i64; 3], u32> = HashMap::new();
        let mut positions = Vec::new();
        let mut index = Vec::with_capacity(mesh.positions.len());
        for &p in &mesh.positions {
            let key = p.map(|v| (v / WELD_STEP).round() as i64);
            let id = *ids.entry(key).or_insert_with(|| {
                positions.push(p);
                (positions.len() - 1) as u32
            });
            index.push(id);
        }
        Self { positions, index }
    }

    /// The triangles that have three distinct corners and a usable normal.
    fn faces(&self, mesh: &Mesh) -> Vec<Face> {
        mesh.indices
            .chunks_exact(3)
            .filter_map(|tri| {
                let verts = [tri[0], tri[1], tri[2]].map(|i| self.index[i as usize]);
                if verts[0] == verts[1] || verts[1] == verts[2] || verts[0] == verts[2] {
                    return None;
                }
                let [a, b, c] = verts.map(|v| self.positions[v as usize]);
                let normal = normalize(cross(sub(b, a), sub(c, a)))?;
                Some(Face { verts, normal })
            })
            .collect()
    }
}

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    (len > 1e-12).then(|| v.map(|c| c / len))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The absolute cosine, so a normal's side does not matter. Feature detection cares about how
/// sharply a surface turns, not which way its triangles wind.
fn abs_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    dot(a, b).abs()
}

fn angle_degrees(a: [f32; 3], b: [f32; 3]) -> f32 {
    abs_dot(a, b).clamp(0.0, 1.0).acos().to_degrees()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Surface;

    /// Appends a quad with the given corners in order, as two triangles.
    fn quad(mesh: &mut Mesh, c: [[f32; 3]; 4]) {
        let first = mesh.positions.len() as u32;
        for p in c {
            mesh.positions.push(p);
            mesh.normals.push([0.0, 0.0, 1.0]);
            mesh.surfaces.push(Surface::Paper);
        }
        mesh.indices
            .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
    }

    /// A box from -1 to 1 with every face split apart, as the stand-in models do.
    fn split_cube() -> Mesh {
        let mut mesh = Mesh::default();
        let (lo, hi) = (-1.0_f32, 1.0_f32);
        let v = |x: f32, y: f32, z: f32| [x, y, z];
        quad(
            &mut mesh,
            [v(lo, lo, lo), v(hi, lo, lo), v(hi, hi, lo), v(lo, hi, lo)],
        );
        quad(
            &mut mesh,
            [v(lo, lo, hi), v(hi, lo, hi), v(hi, hi, hi), v(lo, hi, hi)],
        );
        quad(
            &mut mesh,
            [v(lo, lo, lo), v(hi, lo, lo), v(hi, lo, hi), v(lo, lo, hi)],
        );
        quad(
            &mut mesh,
            [v(lo, hi, lo), v(hi, hi, lo), v(hi, hi, hi), v(lo, hi, hi)],
        );
        quad(
            &mut mesh,
            [v(lo, lo, lo), v(lo, hi, lo), v(lo, hi, hi), v(lo, lo, hi)],
        );
        quad(
            &mut mesh,
            [v(hi, lo, lo), v(hi, hi, lo), v(hi, hi, hi), v(hi, lo, hi)],
        );
        mesh
    }

    /// A prism along y from -1 to 1, whose cross-section is the polyline `profile` in (x, z).
    /// Each profile segment is one flat strip. The caps are left open, so the open boundary
    /// edges are the profile ends.
    fn extrude(profile: &[[f32; 2]]) -> Mesh {
        let mut mesh = Mesh::default();
        for pair in profile.windows(2) {
            let [a, b] = [pair[0], pair[1]];
            quad(
                &mut mesh,
                [
                    [a[0], -1.0, a[1]],
                    [b[0], -1.0, b[1]],
                    [b[0], 1.0, b[1]],
                    [a[0], 1.0, a[1]],
                ],
            );
        }
        mesh
    }

    /// Segments whose two ends sit on the profile point `at` (in x and z) and run along y.
    fn along_y_at(segments: &[[[f32; 3]; 2]], at: [f32; 2]) -> usize {
        segments
            .iter()
            .filter(|s| {
                s.iter()
                    .all(|p| (p[0] - at[0]).abs() < 1e-4 && (p[2] - at[1]).abs() < 1e-4)
                    && (s[0][1] - s[1][1]).abs() > 1.0
            })
            .count()
    }

    #[test]
    fn a_cube_gives_twelve_edges_not_twenty_four_or_thirty_six() {
        let segments = feature_segments(&split_cube());
        assert_eq!(segments.len(), 12, "one line per cube edge: {segments:?}");
        let mut keys: Vec<[[i64; 3]; 2]> = segments
            .iter()
            .map(|s| {
                let mut ends = s.map(|p| p.map(|v| (v * 1000.0).round() as i64));
                ends.sort();
                ends
            })
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 12, "no edge is drawn twice");
    }

    #[test]
    fn a_chamfered_edge_gives_one_bevel_line_and_its_two_ends() {
        let c = 0.5;
        let profile = [
            [-1.0, -1.0],
            [1.0, -1.0],
            [1.0, 1.0 - c],
            [1.0 - c, 1.0],
            [-1.0, 1.0],
            [-1.0, -1.0],
        ];
        let mut mesh = extrude(&profile);
        // Caps at y = -1 and y = 1, each a fan from the first profile point.
        for y in [-1.0_f32, 1.0] {
            let p: Vec<[f32; 3]> = profile[..5].iter().map(|q| [q[0], y, q[1]]).collect();
            for k in 1..4 {
                let first = mesh.positions.len() as u32;
                for corner in [p[0], p[k], p[k + 1]] {
                    mesh.positions.push(corner);
                    mesh.normals.push([0.0, 1.0, 0.0]);
                    mesh.surfaces.push(Surface::Paper);
                }
                mesh.indices.extend([first, first + 1, first + 2]);
            }
        }
        let segments = feature_segments(&mesh);
        assert_eq!(
            along_y_at(&segments, [1.0, 1.0 - c]) + along_y_at(&segments, [1.0 - c, 1.0]),
            1,
            "the chamfer draws one line, not one per side"
        );
        assert_eq!(
            segments.len(),
            14,
            "three box edges, one bevel, and ten cap edges"
        );
    }

    #[test]
    fn a_three_step_bevel_draws_one_line_where_half_its_turn_is() {
        // A wall turning to a top in three 30 degree steps, a 90 degree corner in all.
        let step = |angle: f32| [angle.to_radians().cos(), angle.to_radians().sin()];
        let mut profile = vec![[1.0, 0.0], [1.0, 1.0]];
        let mut at = [1.0_f32, 1.0];
        for angle in [120.0_f32, 150.0, 180.0] {
            let d = step(angle);
            at = [at[0] + 0.5 * d[0], at[1] + 0.5 * d[1]];
            profile.push(at);
        }
        profile.push([at[0] - 2.0, at[1]]);
        let segments = feature_segments(&extrude(&profile));
        assert_eq!(
            along_y_at(&segments, profile[1]),
            0,
            "wall meets the bevel smoothly"
        );
        assert_eq!(
            along_y_at(&segments, profile[3]),
            0,
            "bevel meets the top smoothly"
        );
        assert_eq!(
            along_y_at(&segments, profile[2]),
            1,
            "one line in the bevel"
        );
    }

    #[test]
    fn a_smooth_cylinder_wall_adds_no_lines_only_its_rims() {
        let mut mesh = Mesh::default();
        let segments = 32;
        for k in 0..segments {
            let (s0, c0) = (std::f32::consts::TAU * k as f32 / segments as f32).sin_cos();
            let (s1, c1) = (std::f32::consts::TAU * (k + 1) as f32 / segments as f32).sin_cos();
            quad(
                &mut mesh,
                [[c0, s0, 1.0], [c1, s1, 1.0], [c1, s1, 5.0], [c0, s0, 5.0]],
            );
        }
        let lines = feature_segments(&mesh);
        assert_eq!(lines.len(), 2 * segments, "only the two rims are edges");
    }

    #[test]
    fn a_flat_plate_on_the_ground_has_no_feature_edges() {
        let mut mesh = Mesh::default();
        quad(
            &mut mesh,
            [
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 3.0, 0.0],
                [0.0, 3.0, 0.0],
            ],
        );
        assert!(
            feature_segments(&mesh).is_empty(),
            "ground contact is the outline's job"
        );
    }

    #[test]
    fn feature_lines_are_thin_light_ink_at_their_level() {
        let lines = feature_lines(&split_cube(), Detail::Near);
        assert_eq!(lines.len(), 12);
        assert!(lines.iter().all(|l| l.tone == Ink::Edge
            && l.detail == Detail::Near
            && (l.width - FEATURE_PX).abs() < 1e-6
            && !l.closed
            && l.points.len() == 2));
    }
}
