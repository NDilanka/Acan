//! The kit's isometric camera: azimuth 45 degrees, elevation 30 degrees, scale 1.

/// Projects world points (x and y on the floor, z up) to the iso plane and gives their painter depth.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    sin_a: f32,
    cos_a: f32,
    sin_e: f32,
    cos_e: f32,
    k: f32,
}

impl Camera {
    /// The kit's default camera at scale 1.
    #[must_use]
    pub fn kit() -> Self {
        let azimuth = 45.0_f32.to_radians();
        let elevation = 30.0_f32.to_radians();
        Self {
            sin_a: azimuth.sin(),
            cos_a: azimuth.cos(),
            sin_e: elevation.sin(),
            cos_e: elevation.cos(),
            k: 1.6_f32.sqrt(),
        }
    }

    /// Projects a world point to the iso plane, in plan units with y down.
    #[must_use]
    pub fn project(&self, [x, y, z]: [f32; 3]) -> [f32; 2] {
        [
            (x * self.sin_a - y * self.cos_a) * self.k,
            ((x * self.cos_a + y * self.sin_a) * self.sin_e - z * self.cos_e) * self.k,
        ]
    }

    /// Painter depth of a world point. Larger is nearer the viewer.
    #[must_use]
    pub fn depth(&self, [x, y, z]: [f32; 3]) -> f32 {
        (x * self.cos_a + y * self.sin_a) * self.cos_e + z * self.sin_e
    }

    /// Plan direction toward the viewer, as (x, y).
    pub(crate) fn toward_viewer(&self) -> [f32; 2] {
        [self.cos_a, self.sin_a]
    }

    /// Plan direction the key light comes from, as (x, y). Faces that turn toward it are lighter.
    pub(crate) fn light(&self) -> [f32; 2] {
        [-self.sin_a, self.cos_a]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: [f32; 2], expected: [f32; 2]) -> bool {
        (actual[0] - expected[0]).abs() < 1e-4 && (actual[1] - expected[1]).abs() < 1e-4
    }

    #[test]
    fn projection_matches_the_kit_on_known_points() {
        let cam = Camera::kit();
        assert!(close(cam.project([1.0, 0.0, 0.0]), [0.894_427, 0.447_214]));
        assert!(close(cam.project([0.0, 1.0, 0.0]), [-0.894_427, 0.447_214]));
        assert!(close(cam.project([0.0, 0.0, 1.0]), [0.0, -1.095_445]));
        assert!(close(
            cam.project([2.0, 3.0, 4.0]),
            [-0.894_427, -2.145_712]
        ));
        assert!(close(
            cam.project([-1.5, 2.25, -0.5]),
            [-3.354_102, 0.883_133]
        ));
    }

    #[test]
    fn depth_matches_the_kit_on_known_points() {
        let cam = Camera::kit();
        assert!((cam.depth([1.0, 0.0, 0.0]) - 0.612_372).abs() < 1e-4);
        assert!((cam.depth([0.0, 0.0, 1.0]) - 0.5).abs() < 1e-4);
        assert!((cam.depth([2.0, 3.0, 4.0]) - 5.061_862).abs() < 1e-4);
        assert!((cam.depth([-1.5, 2.25, -0.5]) - 0.209_279).abs() < 1e-4);
    }

    #[test]
    fn every_axis_points_toward_the_viewer() {
        let cam = Camera::kit();
        assert!(cam.depth([1.0, 0.0, 0.0]) > 0.0);
        assert!(cam.depth([0.0, 1.0, 0.0]) > 0.0);
        assert!(cam.depth([0.0, 0.0, 1.0]) > 0.0);
    }
}
