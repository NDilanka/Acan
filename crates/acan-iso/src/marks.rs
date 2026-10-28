//! The marks drawn on the plant: an agent's body, the ring round a waiting agent, and the clash
//! pin. They are models like the machines, so the renderer outlines them with the same line art.
//! Each model's origin is on the ground under it. The renderer picks the colour, from its tint
//! slots and palette, so no model names a hue.

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::model::{Aabb, Ink, Mesh, Model, Surface};
use crate::parts::PLINTH;

/// The height of the plinth top. Agent bodies stand on it, and the ring lies on it.
pub const PLINTH_TOP: f32 = PLINTH;

/// Radius of an agent's capsule body, in plan units.
const BODY_RADIUS: f32 = 4.0;
/// Foot to crown of an agent's body, in plan units. About three times the plinth, so it reads at
/// fit zoom.
pub const BODY_HEIGHT: f32 = 12.0;
const BODY_SEGMENTS: usize = 16;
/// Steps from the equator to the pole of each hemisphere.
const BODY_STEPS: usize = 4;
/// The amber ring round a waiting agent: its inner and outer radius, in plan units.
const RING_INNER: f32 = 5.5;
const RING_OUTER: f32 = 7.5;
// The ring clears the body it surrounds.
const _: () = assert!(RING_INNER > BODY_RADIUS);
const RING_SEGMENTS: usize = 32;
/// The ring sits this far above the plinth top, so it never z-fights the plinth.
const RING_LIFT: f32 = 0.05;
/// The clash pin: its half width at the equator and its height, in plan units.
const PIN_RADIUS: f32 = 3.5;
pub const PIN_HEIGHT: f32 = 11.0;

/// An agent's body: a capsule, shaded as paper so its tint carries the agent's hue.
#[must_use]
pub fn agent_body() -> Model {
    let mut mesh = Mesh::default();
    let (bottom, top) = (BODY_RADIUS, BODY_HEIGHT - BODY_RADIUS);
    let lower = (0..=BODY_STEPS).map(|i| {
        let phi = -FRAC_PI_2 + FRAC_PI_2 * i as f32 / BODY_STEPS as f32;
        (bottom + BODY_RADIUS * phi.sin(), phi)
    });
    let upper = (0..=BODY_STEPS).map(|i| {
        let phi = FRAC_PI_2 * i as f32 / BODY_STEPS as f32;
        (top + BODY_RADIUS * phi.sin(), phi)
    });
    let rings: Vec<(f32, f32)> = lower.chain(upper).collect();
    for &(z, phi) in &rings {
        let (radius, lift) = (BODY_RADIUS * phi.cos(), phi.sin());
        push_ring(
            &mut mesh,
            radius,
            z,
            |theta| [phi.cos() * theta.cos(), phi.cos() * theta.sin(), lift],
            BODY_SEGMENTS,
        );
    }
    band(&mut mesh, rings.len(), BODY_SEGMENTS);
    model_of(mesh)
}

/// A flat amber ring lying on the plinth top round an agent's body.
#[must_use]
pub fn waiting_ring() -> Model {
    let mut mesh = Mesh::default();
    for radius in [RING_OUTER, RING_INNER] {
        push_ring(
            &mut mesh,
            radius,
            RING_LIFT,
            |_| [0.0, 0.0, 1.0],
            RING_SEGMENTS,
        );
    }
    band(&mut mesh, 2, RING_SEGMENTS);
    for surface in &mut mesh.surfaces {
        *surface = Surface::Lit;
    }
    model_of(mesh)
}

/// A red diamond standing on its foot: an octahedron, flat red, so its outline is the only line.
#[must_use]
pub fn clash_pin() -> Model {
    let mut mesh = Mesh::default();
    let mid = PIN_HEIGHT * 0.5;
    for (z, radius) in [(0.0, 0.0), (mid, PIN_RADIUS), (PIN_HEIGHT, 0.0)] {
        for j in 0..4 {
            let theta = TAU * j as f32 / 4.0;
            let position = [radius * theta.cos(), radius * theta.sin(), z];
            let normal = [
                position[0] / PIN_RADIUS,
                position[1] / PIN_RADIUS,
                (z - mid) / mid,
            ];
            mesh.positions.push(position);
            mesh.normals.push(normal);
            mesh.surfaces.push(Surface::Ink(Ink::Red));
        }
    }
    band(&mut mesh, 3, 4);
    model_of(mesh)
}

/// One ring of `segments` vertices at height `z`. `normal` gives the unit normal at each angle.
fn push_ring(
    mesh: &mut Mesh,
    radius: f32,
    z: f32,
    normal: impl Fn(f32) -> [f32; 3],
    segments: usize,
) {
    for j in 0..segments {
        let theta = TAU * j as f32 / segments as f32;
        mesh.positions
            .push([radius * theta.cos(), radius * theta.sin(), z]);
        mesh.normals.push(normal(theta));
        mesh.surfaces.push(Surface::Paper);
    }
}

/// Joins consecutive rings of `segments` vertices into triangles. The rings run in order, each
/// turning counter-clockwise seen from above, so the triangles face outward for rings that climb.
fn band(mesh: &mut Mesh, rings: usize, segments: usize) {
    for k in 0..rings - 1 {
        for j in 0..segments {
            let a = (k * segments + j) as u32;
            let b = (k * segments + (j + 1) % segments) as u32;
            let c = ((k + 1) * segments + j) as u32;
            let d = ((k + 1) * segments + (j + 1) % segments) as u32;
            mesh.indices.extend([a, b, c, b, d, c]);
        }
    }
}

fn model_of(mesh: Mesh) -> Model {
    let mut bounds = Aabb::EMPTY;
    for &p in &mesh.positions {
        bounds.include(p);
    }
    Model {
        meshes: [mesh, Mesh::default(), Mesh::default()],
        lines: Vec::new(),
        dots: Vec::new(),
        bounds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every triangle's normal points away from the model's centre line, so the faces wind outward.
    fn winds_outward(model: &Model) -> bool {
        let mesh = &model.meshes[0];
        let centre = model.bounds.centre();
        mesh.indices.chunks(3).all(|tri| {
            let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| mesh.positions[i as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            let area = n.iter().map(|x| x * x).sum::<f32>();
            if area < 1e-9 {
                return true;
            }
            let out = [
                (a[0] + b[0] + c[0]) / 3.0 - centre[0],
                (a[1] + b[1] + c[1]) / 3.0 - centre[1],
                (a[2] + b[2] + c[2]) / 3.0 - centre[2],
            ];
            n[0] * out[0] + n[1] * out[1] + n[2] * out[2] > 0.0
        })
    }

    #[test]
    fn the_body_stands_on_its_foot_and_is_a_solid() {
        let body = agent_body();
        assert!(
            body.bounds.min[2].abs() < 1e-4,
            "the foot sits on the ground"
        );
        assert!((body.bounds.max[2] - BODY_HEIGHT).abs() < 1e-3);
        assert!(winds_outward(&body), "the capsule faces outward");
    }

    #[test]
    fn the_pin_stands_on_its_foot_and_faces_outward() {
        let pin = clash_pin();
        assert!((pin.bounds.max[2] - PIN_HEIGHT).abs() < 1e-4);
        assert!(winds_outward(&pin), "the diamond faces outward");
    }

    #[test]
    fn the_ring_lies_flat_on_the_plinth_top_round_the_body() {
        let ring = waiting_ring();
        assert!(ring.meshes[0].positions.iter().all(|p| p[2] == RING_LIFT));
        assert!(ring.bounds.max[0] <= RING_OUTER + 1e-4);
        assert!(ring.meshes[0].surfaces.iter().all(|s| *s == Surface::Lit));
    }
}
