//! The orthographic orbit camera. The view turns around a ground point under the screen
//! centre, so orbiting never moves the scene under the pointer.

use crate::model::Aabb;

/// Azimuth of the kit's own view, in degrees.
pub const DEFAULT_AZIMUTH: f32 = 45.0;
/// Elevation of the kit's own view, in degrees.
pub const DEFAULT_ELEVATION: f32 = 30.0;
pub const MIN_ELEVATION: f32 = 15.0;
pub const MAX_ELEVATION: f32 = 80.0;
pub const MIN_ZOOM: f32 = 1e-3;
pub const MAX_ZOOM: f32 = 200.0;

/// The screen axes and the direction toward the viewer, in world space.
///
/// The axes match the kit's projection at every azimuth: the screen right vector is
/// `(sin a, -cos a, 0)` and the viewer sits at `(cos a, sin a)` on the ground. Right and up
/// together point away from the viewer, so the picture is the kit's mirror of a right-handed
/// world. Nothing culls faces, so the mirror changes only which side of a face is shown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Basis {
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub toward: [f32; 3],
}

impl Basis {
    fn at(azimuth_deg: f32, elevation_deg: f32) -> Self {
        let (sin_a, cos_a) = azimuth_deg.to_radians().sin_cos();
        let (sin_e, cos_e) = elevation_deg.to_radians().sin_cos();
        Self {
            right: [sin_a, -cos_a, 0.0],
            up: [-sin_e * cos_a, -sin_e * sin_a, cos_e],
            toward: [cos_e * cos_a, cos_e * sin_a, sin_e],
        }
    }
}

/// The view: a ground pivot, two angles, a zoom and the viewport in device pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pivot: [f32; 2],
    /// Degrees in `[0, 360)`.
    azimuth: f32,
    /// Degrees in `[MIN_ELEVATION, MAX_ELEVATION]`.
    elevation: f32,
    /// Device pixels per plan unit.
    zoom: f32,
    viewport: [f32; 2],
}

impl Camera {
    /// A camera at the kit's angles, looking at the plan origin at zoom 1.
    pub fn new(viewport: [f32; 2]) -> Self {
        Self {
            pivot: [0.0; 2],
            azimuth: DEFAULT_AZIMUTH,
            elevation: DEFAULT_ELEVATION,
            zoom: 1.0,
            viewport,
        }
    }

    pub fn pivot(&self) -> [f32; 2] {
        self.pivot
    }

    /// Moves the ground point under the screen centre.
    pub fn set_pivot(&mut self, pivot: [f32; 2]) {
        self.pivot = pivot;
    }

    pub fn azimuth(&self) -> f32 {
        self.azimuth
    }

    pub fn elevation(&self) -> f32 {
        self.elevation
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn viewport(&self) -> [f32; 2] {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: [f32; 2]) {
        self.viewport = viewport;
    }

    pub fn basis(&self) -> Basis {
        Basis::at(self.azimuth, self.elevation)
    }

    /// Device pixel (y down) of a world point.
    pub fn world_to_screen(&self, p: [f32; 3]) -> [f32; 2] {
        let b = self.basis();
        let d = [p[0] - self.pivot[0], p[1] - self.pivot[1], p[2]];
        [
            self.viewport[0] * 0.5 + self.zoom * dot(d, b.right),
            self.viewport[1] * 0.5 - self.zoom * dot(d, b.up),
        ]
    }

    /// Plan point on the ground under a device pixel.
    pub fn ground_at(&self, px: [f32; 2]) -> [f32; 2] {
        let b = self.basis();
        let sx = (px[0] - self.viewport[0] * 0.5) / self.zoom;
        let sy = (self.viewport[1] * 0.5 - px[1]) / self.zoom;
        // The view ray is pivot + sx * right + sy * up + t * toward. It reaches z = 0 at this t.
        let t = -sy * b.up[2] / b.toward[2];
        [
            self.pivot[0] + sx * b.right[0] + sy * b.up[0] + t * b.toward[0],
            self.pivot[1] + sx * b.right[1] + sy * b.up[1] + t * b.toward[1],
        ]
    }

    /// Moves the view so the content follows a pointer drag of `delta` device pixels.
    pub fn pan_px(&mut self, delta: [f32; 2]) {
        let b = self.basis();
        // Solve for the ground vector v with v . right = -dx / zoom and v . up = dy / zoom.
        let (a11, a12) = (b.right[0], b.right[1]);
        let (a21, a22) = (b.up[0], b.up[1]);
        let (r1, r2) = (-delta[0] / self.zoom, delta[1] / self.zoom);
        let det = a11 * a22 - a12 * a21;
        self.pivot[0] += (r1 * a22 - a12 * r2) / det;
        self.pivot[1] += (a11 * r2 - a21 * r1) / det;
    }

    /// Turns the view by degrees around the ground point at the screen centre.
    pub fn orbit(&mut self, d_azimuth: f32, d_elevation: f32) {
        self.set_angles(self.azimuth + d_azimuth, self.elevation + d_elevation);
    }

    pub fn set_angles(&mut self, azimuth: f32, elevation: f32) {
        self.azimuth = azimuth.rem_euclid(360.0);
        self.elevation = elevation.clamp(MIN_ELEVATION, MAX_ELEVATION);
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Multiplies the zoom by `factor`, keeping the ground point under `px` fixed on screen.
    pub fn zoom_at(&mut self, px: [f32; 2], factor: f32) {
        let before = self.ground_at(px);
        self.set_zoom(self.zoom * factor);
        let after = self.ground_at(px);
        self.pivot[0] += before[0] - after[0];
        self.pivot[1] += before[1] - after[1];
    }

    /// Centres the box and zooms so its projection fits with `margin_px` around it, at the
    /// current angles.
    pub fn fit(&mut self, bounds: &Aabb, margin_px: f32) {
        let (pivot, zoom) = fit_at(
            self.viewport,
            bounds,
            margin_px,
            self.azimuth,
            self.elevation,
        );
        self.pivot = pivot;
        self.zoom = zoom;
    }

    /// Returns to the kit's angles and fits the box.
    pub fn reset_view(&mut self, bounds: &Aabb, margin_px: f32) {
        self.set_angles(DEFAULT_AZIMUTH, DEFAULT_ELEVATION);
        self.fit(bounds, margin_px);
    }
}

/// Pivot and zoom that frame `bounds` in `viewport` at the given angles.
pub fn fit_at(
    viewport: [f32; 2],
    bounds: &Aabb,
    margin_px: f32,
    azimuth: f32,
    elevation: f32,
) -> ([f32; 2], f32) {
    let b = Basis::at(azimuth, elevation);
    let centre = bounds.centre();
    let (mut half_right, mut half_up) = (0.0_f32, 0.0_f32);
    for corner in bounds.corners() {
        let d = sub(corner, centre);
        half_right = half_right.max(dot(d, b.right).abs());
        half_up = half_up.max(dot(d, b.up).abs());
    }
    let avail_w = (viewport[0] - 2.0 * margin_px).max(1.0);
    let avail_h = (viewport[1] - 2.0 * margin_px).max(1.0);
    let zoom = (avail_w / (2.0 * half_right.max(1e-3)))
        .min(avail_h / (2.0 * half_up.max(1e-3)))
        .clamp(MIN_ZOOM, MAX_ZOOM);
    // The ground point on the view ray through the box centre, so the centre lands mid-screen.
    let t = -centre[2] / b.toward[2];
    let pivot = [centre[0] + t * b.toward[0], centre[1] + t * b.toward[1]];
    (pivot, zoom)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-2 && (a[1] - b[1]).abs() < 1e-2
    }

    fn block() -> Aabb {
        Aabb {
            min: [-120.0, -60.0, 0.0],
            max: [260.0, 180.0, 60.0],
        }
    }

    #[test]
    fn default_view_matches_the_kit_projection() {
        let cam = Camera::new([800.0, 600.0]);
        let b = cam.basis();
        let x = [1.0, 0.0, 0.0];
        let y_up = dot(x, b.up);
        let x_right = dot(x, b.right);
        assert!((x_right - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
        assert!((y_up + 0.353_553).abs() < 1e-4, "kit x goes right and down");
        let z = [0.0, 0.0, 1.0];
        assert!((dot(z, b.up) - 0.866_025).abs() < 1e-4);
        assert!((dot(x, b.toward) - 0.612_372).abs() < 1e-4);
    }

    #[test]
    fn orbit_keeps_the_pivot_at_the_screen_centre() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.pan_px([130.0, -55.0]);
        cam.zoom_at([300.0, 200.0], 1.7);
        let pivot = cam.pivot();
        let before = cam.world_to_screen([pivot[0], pivot[1], 0.0]);
        for step in 0..36 {
            cam.orbit(10.0, if step % 2 == 0 { 4.0 } else { -3.0 });
            let after = cam.world_to_screen([pivot[0], pivot[1], 0.0]);
            assert!(
                close(after, [400.0, 300.0]),
                "pivot left the centre at step {step}"
            );
        }
        assert!(close(before, [400.0, 300.0]));
    }

    #[test]
    fn azimuth_wraps_through_three_sixty() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.set_angles(0.0, 30.0);
        cam.orbit(-60.0, 0.0);
        assert!((cam.azimuth() - 300.0).abs() < 1e-4);
        cam.orbit(120.0, 0.0);
        assert!((cam.azimuth() - 60.0).abs() < 1e-4);
        let start = cam.azimuth();
        for _ in 0..8 {
            cam.orbit(45.0, 0.0);
            assert!(cam.azimuth() >= 0.0 && cam.azimuth() < 360.0);
        }
        assert!(
            (cam.azimuth() - start).abs() < 1e-3,
            "eight 45 degree turns close the circle"
        );
    }

    #[test]
    fn elevation_clamps_between_fifteen_and_eighty() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.orbit(0.0, 500.0);
        assert!((cam.elevation() - MAX_ELEVATION).abs() < 1e-4);
        cam.orbit(0.0, -500.0);
        assert!((cam.elevation() - MIN_ELEVATION).abs() < 1e-4);
    }

    #[test]
    fn pan_moves_content_with_the_pointer_at_every_angle() {
        for azimuth in [0.0_f32, 45.0, 135.0, 250.0] {
            let mut cam = Camera::new([800.0, 600.0]);
            cam.set_angles(azimuth, 40.0);
            cam.set_zoom(2.5);
            let p = [30.0, -20.0, 0.0];
            let before = cam.world_to_screen(p);
            cam.pan_px([25.0, -12.0]);
            let after = cam.world_to_screen(p);
            assert!(
                close(after, [before[0] + 25.0, before[1] - 12.0]),
                "azimuth {azimuth}: {before:?} -> {after:?}"
            );
        }
    }

    #[test]
    fn zoom_keeps_the_anchor_fixed() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.set_angles(120.0, 55.0);
        let anchor = [610.0, 95.0];
        let ground = cam.ground_at(anchor);
        cam.zoom_at(anchor, 3.0);
        let back = cam.world_to_screen([ground[0], ground[1], 0.0]);
        assert!(close(back, anchor));
        assert!((cam.zoom() - 3.0).abs() < 1e-4);
    }

    #[test]
    fn ground_at_inverts_world_to_screen_on_the_ground() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.set_angles(200.0, 65.0);
        cam.set_zoom(0.8);
        let p = [77.0, -33.0];
        let px = cam.world_to_screen([p[0], p[1], 0.0]);
        let back = cam.ground_at(px);
        assert!((back[0] - p[0]).abs() < 1e-2 && (back[1] - p[1]).abs() < 1e-2);
    }

    #[test]
    fn fit_contains_all_bounds_at_any_azimuth() {
        let bounds = block();
        let margin = 24.0;
        let mut cam = Camera::new([1440.0, 900.0]);
        for azimuth in (0..360).step_by(15) {
            for elevation in [15.0_f32, 30.0, 60.0, 80.0] {
                cam.set_angles(azimuth as f32, elevation);
                cam.fit(&bounds, margin);
                for corner in bounds.corners() {
                    let s = cam.world_to_screen(corner);
                    assert!(
                        s[0] >= margin - 0.5
                            && s[0] <= 1440.0 - margin + 0.5
                            && s[1] >= margin - 0.5
                            && s[1] <= 900.0 - margin + 0.5,
                        "corner {corner:?} at {s:?} outside the margin, azimuth {azimuth}, elevation {elevation}"
                    );
                }
            }
        }
    }

    #[test]
    fn reset_view_restores_the_kit_angles() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.set_angles(300.0, 70.0);
        cam.reset_view(&block(), 24.0);
        assert!((cam.azimuth() - DEFAULT_AZIMUTH).abs() < 1e-4);
        assert!((cam.elevation() - DEFAULT_ELEVATION).abs() < 1e-4);
    }
}
