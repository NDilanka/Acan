//! The orbit camera. The view turns around a ground point under the screen centre, so orbiting
//! never moves the scene under the pointer. It projects in perspective, like Blender's default
//! viewport, or orthographically. Both share one pivot, two angles and one zoom, so switching
//! projection keeps the framing and the scale at the pivot.

use crate::model::Aabb;
use crate::theme;

/// Azimuth of the kit's own view, in degrees.
pub const DEFAULT_AZIMUTH: f32 = 45.0;
/// Elevation of the kit's own view, in degrees.
pub const DEFAULT_ELEVATION: f32 = 30.0;
/// Vertical field of view of the perspective camera, in degrees.
pub const VERTICAL_FOV: f32 = 35.0;
/// The lowest tilt of each projection. A perspective view can sit nearly flat and still show
/// the horizon. An orthographic view that flat squashes the plant into a strip.
pub const MIN_ELEVATION_PERSPECTIVE: f32 = 5.0;
pub const MIN_ELEVATION_ORTHOGRAPHIC: f32 = 15.0;
pub const MAX_ELEVATION: f32 = 89.0;
pub const MIN_ZOOM: f32 = 1e-3;
pub const MAX_ZOOM: f32 = 200.0;
/// In a low perspective view the horizon sits no lower than this fraction of the viewport
/// from the top, so empty sky never takes more of the frame than that.
pub const HORIZON_FROM_TOP: f32 = 0.2;
/// The near plane, as a fraction of the eye distance.
pub const NEAR_FRACTION: f32 = 0.01;
/// The orthographic far plane, as a multiple of the eye distance. Every visible ground point at
/// the lowest orthographic elevation lies inside it. The perspective far plane is at infinity.
pub const ORTHO_FAR: f32 = 4.0;

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

/// How the scene is projected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Projection {
    #[default]
    Perspective,
    Orthographic,
}

/// The view: a ground pivot, a centre height, two angles, a zoom, a projection and the viewport in
/// device pixels.
///
/// The zoom is device pixels per plan unit at the view centre in both projections. The perspective
/// eye sits `eye_distance` from the view centre, which is the distance at which the centre's scale
/// is `zoom` for the vertical field of view. The orthographic view uses the same eye distance for
/// fog, so the two projections look the same.
/// Extra room a fit keeps at the top, right and bottom edges, in device pixels, for the HTML
/// the page draws over the plant there: flag chips above their drones and to the right of their
/// anchors, machine names below it. The fit's left edge keeps the plain margin. The room is in
/// pixels because the overlays are HTML: their size does not change with the zoom.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FitRoom {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pivot: [f32; 2],
    /// Height of the view centre above the pivot's ground point, in plan units. The centre is
    /// the point this high on the view axis. Zero puts the centre on the ground at the pivot.
    height: f32,
    /// Degrees in `[0, 360)`.
    azimuth: f32,
    /// Degrees, clamped to the range of the projection.
    elevation: f32,
    zoom: f32,
    viewport: [f32; 2],
    projection: Projection,
    /// The box the fog keeps clear. Empty until a scene is set.
    fog_bounds: Aabb,
}

impl Camera {
    /// A perspective camera at the kit's angles, looking at the plan origin at zoom 1.
    pub fn new(viewport: [f32; 2]) -> Self {
        Self {
            pivot: [0.0; 2],
            height: 0.0,
            azimuth: DEFAULT_AZIMUTH,
            elevation: DEFAULT_ELEVATION,
            zoom: 1.0,
            viewport,
            projection: Projection::Perspective,
            fog_bounds: Aabb::EMPTY,
        }
    }

    pub fn pivot(&self) -> [f32; 2] {
        self.pivot
    }

    /// Moves the ground point under the screen centre, and puts the view centre on the ground.
    pub fn set_pivot(&mut self, pivot: [f32; 2]) {
        self.pivot = pivot;
        self.height = 0.0;
    }

    /// Centres the view on the world point `p`, so `p` projects to the screen centre. The
    /// pivot becomes the ground point on the ray through `p`, and the view centre rises to `p`.
    /// The eye then sits the eye distance from `p`, so a point in the air frames at the same
    /// scale as the ground beneath it, and a zoom sets its scale.
    pub fn centre_on(&mut self, p: [f32; 3]) {
        let toward = self.basis().toward;
        let t = -p[2] / toward[2];
        self.pivot = [p[0] + t * toward[0], p[1] + t * toward[1]];
        self.height = p[2];
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

    pub fn projection(&self) -> Projection {
        self.projection
    }

    /// Switches projection. The angles are clamped to the new projection's range, and the
    /// pivot and zoom stay, so the framing holds.
    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.elevation = self.elevation.clamp(self.min_elevation(), MAX_ELEVATION);
    }

    pub fn toggle_projection(&mut self) {
        let next = match self.projection {
            Projection::Perspective => Projection::Orthographic,
            Projection::Orthographic => Projection::Perspective,
        };
        self.set_projection(next);
    }

    pub fn min_elevation(&self) -> f32 {
        match self.projection {
            Projection::Perspective => MIN_ELEVATION_PERSPECTIVE,
            Projection::Orthographic => MIN_ELEVATION_ORTHOGRAPHIC,
        }
    }

    pub fn basis(&self) -> Basis {
        Basis::at(self.azimuth, self.elevation)
    }

    /// Distance from the eye to the view centre, in plan units. Zoom changes it: a dolly in is a
    /// shorter distance at the same field of view.
    pub fn eye_distance(&self) -> f32 {
        self.viewport[1] * 0.5 / (self.zoom * half_fov_tan())
    }

    /// The view centre: the pivot's ground point lifted `height` up the view axis.
    fn centre(&self) -> [f32; 3] {
        let b = self.basis();
        let s = self.height / b.toward[2];
        [
            self.pivot[0] + s * b.toward[0],
            self.pivot[1] + s * b.toward[1],
            self.height,
        ]
    }

    /// The perspective eye. For the orthographic view it is the equivalent eye, used for fog.
    pub fn eye(&self) -> [f32; 3] {
        let b = self.basis();
        let c = self.centre();
        let d = self.eye_distance();
        [
            c[0] + d * b.toward[0],
            c[1] + d * b.toward[1],
            c[2] + d * b.toward[2],
        ]
    }

    /// The device-pixel row the view axis, and so the pivot, projects to. It is the middle of
    /// the viewport, except in a low perspective view, where it rises just enough to lift the
    /// horizon to `HORIZON_FROM_TOP`. The focal length does not change with zoom, so neither
    /// does this.
    pub fn axis_y(&self) -> f32 {
        let vh = self.viewport[1];
        let middle = vh * 0.5;
        if self.projection == Projection::Orthographic {
            return middle;
        }
        let horizon = middle - self.focal() * self.elevation.to_radians().tan();
        middle - (horizon - HORIZON_FROM_TOP * vh).max(0.0)
    }

    /// The world point on the view ray through device pixel `px` at `depth` plan units in
    /// front of the eye.
    pub fn ray_point(&self, px: [f32; 2], depth: f32) -> [f32; 3] {
        let b = self.basis();
        let eye = self.eye();
        let sx = px[0] - self.viewport[0] * 0.5;
        let (side, rise) = match self.projection {
            Projection::Perspective => {
                let scale = depth / self.focal();
                (sx * scale, (self.axis_y() - px[1]) * scale)
            }
            Projection::Orthographic => {
                (sx / self.zoom, (self.viewport[1] * 0.5 - px[1]) / self.zoom)
            }
        };
        std::array::from_fn(|axis| {
            eye[axis] - depth * b.toward[axis] + side * b.right[axis] + rise * b.up[axis]
        })
    }

    /// Focal length in device pixels: the pivot's scale times the eye distance.
    pub fn focal(&self) -> f32 {
        self.zoom * self.eye_distance()
    }

    /// Device pixel (y down) of a world point. Points behind the eye land on garbage, so
    /// callers project only points in front of the camera.
    pub fn world_to_screen(&self, p: [f32; 3]) -> [f32; 2] {
        self.projector().world_to_screen(p)
    }

    /// The camera's projection worked out once, for projecting many points.
    pub fn projector(&self) -> Projector {
        Projector {
            basis: self.basis(),
            eye: self.eye(),
            pivot: self.pivot,
            projection: self.projection,
            zoom: self.zoom,
            focal: self.focal(),
            centre: [self.viewport[0] * 0.5, self.viewport[1] * 0.5],
            axis_y: self.axis_y(),
            near: NEAR_FRACTION * self.eye_distance(),
        }
    }

    /// Plan point on the ground under a device pixel, or `None` where the pixel's ray rises
    /// above the horizon.
    pub fn ground_at(&self, px: [f32; 2]) -> Option<[f32; 2]> {
        let b = self.basis();
        let [vw, vh] = self.viewport;
        let sx = px[0] - vw * 0.5;
        let sy = match self.projection {
            Projection::Perspective => self.axis_y() - px[1],
            Projection::Orthographic => vh * 0.5 - px[1],
        };
        match self.projection {
            Projection::Perspective => {
                let eye = self.eye();
                let f = self.focal();
                let dir: [f32; 3] = std::array::from_fn(|axis| {
                    -f * b.toward[axis] + sx * b.right[axis] + sy * b.up[axis]
                });
                if dir[2] >= 0.0 {
                    return None;
                }
                let t = -eye[2] / dir[2];
                Some([eye[0] + t * dir[0], eye[1] + t * dir[1]])
            }
            Projection::Orthographic => {
                let sx = sx / self.zoom;
                let sy = sy / self.zoom;
                let t = -sy * b.up[2] / b.toward[2];
                Some([
                    self.pivot[0] + sx * b.right[0] + sy * b.up[0] + t * b.toward[0],
                    self.pivot[1] + sx * b.right[1] + sy * b.up[1] + t * b.toward[1],
                ])
            }
        }
    }

    /// Moves the view so the ground point under `from` is under `to`. The pointer drags the
    /// scene exactly, in either projection.
    pub fn drag_ground(&mut self, from: [f32; 2], to: [f32; 2]) {
        let (Some(start), Some(now)) = (self.ground_at(from), self.ground_at(to)) else {
            return;
        };
        self.pivot[0] += start[0] - now[0];
        self.pivot[1] += start[1] - now[1];
    }

    /// Turns the view by degrees around the ground point at the screen centre.
    pub fn orbit(&mut self, d_azimuth: f32, d_elevation: f32) {
        self.set_angles(self.azimuth + d_azimuth, self.elevation + d_elevation);
    }

    pub fn set_angles(&mut self, azimuth: f32, elevation: f32) {
        self.azimuth = azimuth.rem_euclid(360.0);
        self.elevation = elevation.clamp(self.min_elevation(), MAX_ELEVATION);
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Multiplies the zoom by `factor`, keeping the ground point under `px` fixed on screen.
    /// Above the horizon the zoom turns about the pivot.
    pub fn zoom_at(&mut self, px: [f32; 2], factor: f32) {
        let before = self.ground_at(px).unwrap_or(self.pivot);
        self.set_zoom(self.zoom * factor);
        let after = self.ground_at(px).unwrap_or(self.pivot);
        self.pivot[0] += before[0] - after[0];
        self.pivot[1] += before[1] - after[1];
    }

    /// Centres the box and zooms so every corner projects inside `margin_px`, at the current
    /// angles and projection. The projected corners end up centred, which the 3D centre alone
    /// does not give under perspective.
    pub fn fit(&mut self, bounds: &Aabb, margin_px: f32) {
        self.fit_boxes(std::slice::from_ref(bounds), margin_px);
    }

    /// Centres several boxes and zooms so every corner of every box projects inside `margin_px`.
    /// The boxes' corners are what count, so a few small boxes frame the parts of a scene that
    /// matter, and the empty space between them does not push the picture off centre. Nothing
    /// changes when `boxes` is empty.
    pub fn fit_boxes(&mut self, boxes: &[Aabb], margin_px: f32) {
        self.fit_boxes_room(boxes, margin_px, FitRoom::default());
    }

    /// Like [`fit_boxes`], but the top, right and bottom edges keep `room` of extra space, for
    /// what the page draws over the plant there. The boxes' middle lands on the centre of the
    /// region inside the margins, so the extra room stays empty.
    pub fn fit_boxes_room(&mut self, boxes: &[Aabb], margin_px: f32, room: FitRoom) {
        const PASSES: usize = 4;
        if boxes.is_empty() {
            return;
        }
        let mut union = Aabb::EMPTY;
        for bounds in boxes {
            union.include(bounds.min);
            union.include(bounds.max);
        }
        let corners: Vec<[f32; 3]> = boxes.iter().flat_map(|bounds| bounds.corners()).collect();
        let b = self.basis();
        let centre = union.centre();
        // The ground point on the view ray through the box centre, so the centre lands mid-screen.
        // A fit centres on the ground, so the view centre is back on the ground too.
        let t = -centre[2] / b.toward[2];
        self.pivot = [centre[0] + t * b.toward[0], centre[1] + t * b.toward[1]];
        self.height = 0.0;
        for _ in 0..PASSES {
            self.zoom = self.largest_zoom_fitting(&corners, margin_px, room);
            self.centre_projected(&corners, centre, room);
        }
        self.zoom = self.largest_zoom_fitting(&corners, margin_px, room);
    }

    /// Moves the pivot so the middle of the projected corners lands on the centre of the region
    /// inside the margins, with `room` of extra space at the top, right and bottom. A pivot shift `v`
    /// moves a point's screen position by `scale * (-v·right, v·up)`, where `scale` is the focal
    /// length over the depth at `centre` in perspective. The shift is solved from that linear
    /// model, and the next pass corrects the error it leaves.
    fn centre_projected(&mut self, corners: &[[f32; 3]], centre: [f32; 3], room: FitRoom) {
        let [vw, vh] = self.viewport;
        let mut lo = [f32::INFINITY; 2];
        let mut hi = [f32::NEG_INFINITY; 2];
        for &corner in corners {
            let s = self.world_to_screen(corner);
            for axis in 0..2 {
                lo[axis] = lo[axis].min(s[axis]);
                hi[axis] = hi[axis].max(s[axis]);
            }
        }
        let off = [
            (lo[0] + hi[0]) * 0.5 - (vw - room.right) * 0.5,
            (lo[1] + hi[1]) * 0.5 - (vh + room.top - room.bottom) * 0.5,
        ];
        let basis = self.basis();
        let scale = match self.projection {
            Projection::Perspective => {
                let depth = -dot(sub(centre, self.eye()), basis.toward);
                self.focal() / depth
            }
            Projection::Orthographic => self.zoom,
        };
        if !scale.is_finite() || scale <= 0.0 {
            return;
        }
        // Solve dot(v, right) = off_x / scale and dot(v, up) = -off_y / scale for the ground
        // shift `v`. The determinant is -sin(elevation), so it is nonzero above the minimum tilt.
        let (a, b) = (basis.right[0], basis.right[1]);
        let (c, d) = (basis.up[0], basis.up[1]);
        let (p, q) = (off[0] / scale, -off[1] / scale);
        let det = a * d - b * c;
        if det.abs() < 1e-6 {
            return;
        }
        self.pivot[0] += (p * d - b * q) / det;
        self.pivot[1] += (a * q - c * p) / det;
    }

    /// Returns to the kit's angles and fits the box.
    pub fn reset_view(&mut self, bounds: &Aabb, margin_px: f32) {
        self.set_angles(DEFAULT_AZIMUTH, DEFAULT_ELEVATION);
        self.fit(bounds, margin_px);
    }

    /// The largest zoom at which every one of `corners` projects inside the margin, with
    /// `room` of extra space at the top, right and bottom, found by bisection on the geometric
    /// scale. The corners move monotonically with zoom.
    fn largest_zoom_fitting(&self, corners: &[[f32; 3]], margin_px: f32, room: FitRoom) -> f32 {
        let mut probe = *self;
        let [vw, vh] = self.viewport;
        let fits = |zoom: f32, probe: &mut Camera| {
            probe.set_zoom(zoom);
            let toward = probe.basis().toward;
            let eye = probe.eye();
            corners.iter().all(|&corner| {
                let in_front = match probe.projection() {
                    Projection::Perspective => dot(sub(corner, eye), toward) < 0.0,
                    Projection::Orthographic => true,
                };
                let s = probe.world_to_screen(corner);
                let inside = |v: f32, lo: f32, hi: f32| v >= lo - 0.5 && v <= hi + 0.5;
                in_front
                    && inside(s[0], margin_px, vw - margin_px - room.right)
                    && inside(s[1], margin_px + room.top, vh - margin_px - room.bottom)
            })
        };
        let (mut lo, mut hi) = (MIN_ZOOM, MAX_ZOOM);
        if fits(hi, &mut probe) {
            return hi;
        }
        for _ in 0..60 {
            let mid = (lo * hi).sqrt();
            if fits(mid, &mut probe) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    }

    /// Keeps the fog clear of `bounds`, so it starts past the scene's far corner.
    pub fn set_fog_bounds(&mut self, bounds: Aabb) {
        self.fog_bounds = bounds;
    }

    /// Where the fog starts and ends, in plan units from the eye. See `theme::FOG_NEAR`.
    pub fn fog_range(&self) -> (f32, f32) {
        let eye = self.eye();
        let b = self.fog_bounds;
        let far_corner = if b.min[0] <= b.max[0] {
            b.corners()
                .iter()
                .map(|&c| distance(c, eye))
                .fold(0.0, f32::max)
        } else {
            0.0
        };
        let start = (theme::FOG_NEAR * self.eye_distance()).max(far_corner);
        (start, start * theme::FOG_SPAN)
    }

    /// Fog factor of a world point: 0 before the fog starts and 1 past its end.
    pub fn fog_at(&self, p: [f32; 3]) -> f32 {
        let (start, end) = self.fog_range();
        smoothstep(start, end, distance(p, self.eye()))
    }

    /// Whether a world point lies beyond the near plane, so it projects to a real screen spot.
    pub fn in_front(&self, p: [f32; 3]) -> bool {
        self.projector().in_front(p)
    }
}

/// A snapshot of a camera's projection. It goes stale when the camera moves.
#[derive(Clone, Copy, Debug)]
pub struct Projector {
    basis: Basis,
    eye: [f32; 3],
    pivot: [f32; 2],
    projection: Projection,
    zoom: f32,
    focal: f32,
    /// The viewport's middle, in device pixels.
    centre: [f32; 2],
    axis_y: f32,
    /// The near plane's distance from the eye.
    near: f32,
}

impl Projector {
    /// See `Camera::world_to_screen`.
    pub fn world_to_screen(&self, p: [f32; 3]) -> [f32; 2] {
        let b = &self.basis;
        match self.projection {
            Projection::Perspective => {
                let rel = sub(p, self.eye);
                let depth = -dot(rel, b.toward);
                let f = self.focal / depth;
                [
                    self.centre[0] + f * dot(rel, b.right),
                    self.axis_y - f * dot(rel, b.up),
                ]
            }
            Projection::Orthographic => {
                let d = [p[0] - self.pivot[0], p[1] - self.pivot[1], p[2]];
                [
                    self.centre[0] + self.zoom * dot(d, b.right),
                    self.centre[1] - self.zoom * dot(d, b.up),
                ]
            }
        }
    }

    /// See `Camera::in_front`.
    pub fn in_front(&self, p: [f32; 3]) -> bool {
        -dot(sub(p, self.eye), self.basis.toward) > self.near
    }
}

fn half_fov_tan() -> f32 {
    (VERTICAL_FOV * 0.5).to_radians().tan()
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    dot(sub(a, b), sub(a, b)).sqrt()
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

    fn both_projections() -> [Projection; 2] {
        [Projection::Perspective, Projection::Orthographic]
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
    fn ground_round_trips_in_both_projections() {
        for projection in both_projections() {
            for (azimuth, elevation, zoom) in
                [(0.0, 5.0, 0.7), (135.0, 30.0, 2.5), (250.0, 80.0, 9.0)]
            {
                let mut cam = Camera::new([1440.0, 900.0]);
                cam.set_projection(projection);
                cam.set_angles(azimuth, elevation);
                cam.set_zoom(zoom);
                cam.set_pivot([40.0, -25.0]);
                for p in [[77.0, -33.0], [-300.0, 120.0], [40.0, -25.0]] {
                    let px = cam.world_to_screen([p[0], p[1], 0.0]);
                    let back = cam.ground_at(px).expect("ground in view");
                    assert!(
                        (back[0] - p[0]).abs() < 1e-2 && (back[1] - p[1]).abs() < 1e-2,
                        "{projection:?} at {azimuth}/{elevation}: {p:?} came back as {back:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_horizon_has_no_ground_point() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_angles(45.0, 5.0);
        cam.set_zoom(0.4);
        assert_eq!(cam.ground_at([720.0, 0.0]), None, "the top row sees sky");
        assert!(cam.ground_at([720.0, 899.0]).is_some());
    }

    #[test]
    fn orbit_keeps_the_pivot_at_the_screen_centre() {
        for projection in both_projections() {
            let mut cam = Camera::new([800.0, 600.0]);
            cam.set_projection(projection);
            cam.zoom_at([300.0, 200.0], 1.7);
            cam.drag_ground([300.0, 200.0], [420.0, 255.0]);
            let pivot = cam.pivot();
            for step in 0..36 {
                cam.orbit(10.0, if step % 2 == 0 { 4.0 } else { -3.0 });
                let after = cam.world_to_screen([pivot[0], pivot[1], 0.0]);
                assert!(
                    close(after, [400.0, 300.0]),
                    "{projection:?}: pivot left the centre at step {step}"
                );
            }
        }
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
    fn elevation_clamps_to_the_range_of_each_projection() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.orbit(0.0, 500.0);
        assert!((cam.elevation() - MAX_ELEVATION).abs() < 1e-4);
        cam.orbit(0.0, -500.0);
        assert!((cam.elevation() - MIN_ELEVATION_PERSPECTIVE).abs() < 1e-4);
        cam.set_projection(Projection::Orthographic);
        assert!((cam.elevation() - MIN_ELEVATION_ORTHOGRAPHIC).abs() < 1e-4);
        cam.orbit(0.0, -500.0);
        assert!((cam.elevation() - MIN_ELEVATION_ORTHOGRAPHIC).abs() < 1e-4);
    }

    #[test]
    fn the_key_toggles_between_the_two_projections() {
        let mut cam = Camera::new([800.0, 600.0]);
        assert_eq!(cam.projection(), Projection::Perspective);
        cam.toggle_projection();
        assert_eq!(cam.projection(), Projection::Orthographic);
        cam.toggle_projection();
        assert_eq!(cam.projection(), Projection::Perspective);
    }

    #[test]
    fn toggling_keeps_the_pivot_and_the_scale_there() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_angles(120.0, 40.0);
        cam.set_zoom(2.2);
        cam.set_pivot([12.0, 34.0]);
        for projection in both_projections() {
            cam.set_projection(projection);
            let pivot = cam.pivot();
            let centre = cam.world_to_screen([pivot[0], pivot[1], 0.0]);
            assert!(close(centre, [720.0, 450.0]), "{projection:?}: {centre:?}");
            let b = cam.basis();
            let one_right =
                cam.world_to_screen([pivot[0] + b.right[0], pivot[1] + b.right[1], b.right[2]]);
            assert!(
                ((one_right[0] - centre[0]) - 2.2).abs() < 1e-3,
                "{projection:?} scale at the pivot is the zoom"
            );
        }
    }

    #[test]
    fn the_orthographic_half_height_is_the_eye_distance_times_the_tangent() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_zoom(3.0);
        let half_height = cam.viewport()[1] * 0.5 / cam.zoom();
        let expected = cam.eye_distance() * half_fov_tan();
        assert!((half_height - expected).abs() < 1e-3);
    }

    #[test]
    fn drag_keeps_the_ground_under_the_pointer_in_both_projections() {
        for projection in both_projections() {
            for azimuth in [0.0_f32, 45.0, 135.0, 250.0] {
                let mut cam = Camera::new([1440.0, 900.0]);
                cam.set_projection(projection);
                cam.set_angles(azimuth, 40.0);
                cam.set_zoom(2.5);
                let from = [300.0, 620.0];
                let to = [340.0, 600.0];
                let grabbed = cam.ground_at(from).expect("ground under the grab");
                cam.drag_ground(from, to);
                let under = cam.world_to_screen([grabbed[0], grabbed[1], 0.0]);
                assert!(
                    close(under, to),
                    "{projection:?} azimuth {azimuth}: grabbed point at {under:?}, pointer at {to:?}"
                );
            }
        }
    }

    #[test]
    fn zoom_keeps_the_anchor_fixed_in_both_projections() {
        for projection in both_projections() {
            let mut cam = Camera::new([800.0, 600.0]);
            cam.set_projection(projection);
            cam.set_angles(120.0, 55.0);
            let anchor = [610.0, 395.0];
            let ground = cam.ground_at(anchor).expect("ground under the anchor");
            cam.zoom_at(anchor, 3.0);
            let back = cam.world_to_screen([ground[0], ground[1], 0.0]);
            assert!(close(back, anchor), "{projection:?}: {back:?}");
            assert!((cam.zoom() - 3.0).abs() < 1e-4);
        }
    }

    #[test]
    fn fit_contains_all_bounds_in_both_projections_at_any_azimuth() {
        let bounds = block();
        let margin = 24.0;
        let mut cam = Camera::new([1440.0, 900.0]);
        for projection in both_projections() {
            cam.set_projection(projection);
            for azimuth in (0..360).step_by(15) {
                for elevation in [5.0_f32, 15.0, 30.0, 60.0, 89.0] {
                    cam.set_angles(azimuth as f32, elevation);
                    cam.fit(&bounds, margin);
                    for corner in bounds.corners() {
                        let s = cam.world_to_screen(corner);
                        assert!(
                            s[0] >= margin - 0.5
                                && s[0] <= 1440.0 - margin + 0.5
                                && s[1] >= margin - 0.5
                                && s[1] <= 900.0 - margin + 0.5,
                            "{projection:?}: corner {corner:?} at {s:?}, azimuth {azimuth}, elevation {elevation}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn fit_touches_the_margin_and_centres_the_projected_box() {
        let bounds = block();
        let margin = 24.0;
        let viewport = [1440.0_f32, 900.0];
        let mut cam = Camera::new(viewport);
        for projection in both_projections() {
            cam.set_projection(projection);
            cam.set_angles(DEFAULT_AZIMUTH, DEFAULT_ELEVATION);
            cam.fit(&bounds, margin);
            let screen: Vec<[f32; 2]> = bounds
                .corners()
                .iter()
                .map(|&c| cam.world_to_screen(c))
                .collect();
            let lo =
                [0, 1].map(|axis| screen.iter().map(|s| s[axis]).fold(f32::INFINITY, f32::min));
            let hi = [0, 1].map(|axis| {
                screen
                    .iter()
                    .map(|s| s[axis])
                    .fold(f32::NEG_INFINITY, f32::max)
            });
            let gap = [
                lo[0] - margin,
                viewport[0] - margin - hi[0],
                lo[1] - margin,
                viewport[1] - margin - hi[1],
            ]
            .into_iter()
            .fold(f32::INFINITY, f32::min);
            assert!(
                gap < 1.0,
                "{projection:?}: no side reaches the margin, gap {gap}"
            );
            for axis in 0..2 {
                let off = (lo[axis] + hi[axis]) * 0.5 - viewport[axis] * 0.5;
                assert!(
                    off.abs() < 2.0,
                    "{projection:?}: axis {axis} off centre by {off}"
                );
            }
        }
    }

    #[test]
    fn a_low_view_lifts_the_horizon_and_keeps_screen_and_ground_in_step() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_angles(DEFAULT_AZIMUTH, 6.0);
        cam.fit(&block(), 24.0);
        let b = cam.basis();
        let far = 1e6;
        let horizon = cam.world_to_screen([
            cam.eye()[0] - far * b.toward[0] / b.toward[0].hypot(b.toward[1]),
            cam.eye()[1] - far * b.toward[1] / b.toward[0].hypot(b.toward[1]),
            cam.eye()[2],
        ]);
        assert!(
            (horizon[1] - HORIZON_FROM_TOP * 900.0).abs() < 2.0,
            "horizon at {}",
            horizon[1]
        );
        let p = [30.0, -20.0, 0.0];
        let s = cam.world_to_screen(p);
        let back = cam.ground_at(s).expect("ground");
        assert!(close(back, [p[0], p[1]]), "{back:?}");
        let depth = -dot(sub(p, cam.eye()), b.toward);
        let on_ray = cam.ray_point(s, depth);
        assert!(distance(on_ray, p) < 1e-2, "{on_ray:?}");
        cam.set_angles(DEFAULT_AZIMUTH, DEFAULT_ELEVATION);
        assert_eq!(
            cam.axis_y(),
            450.0,
            "a normal view keeps the axis mid-screen"
        );
    }

    #[test]
    fn fit_boxes_centres_the_boxes_and_ignores_the_space_between_them() {
        // Two small boxes at opposite corners of a wide plan, so their union is mostly empty.
        let near = Aabb {
            min: [-200.0, -100.0, 0.0],
            max: [-180.0, -80.0, 20.0],
        };
        let far = Aabb {
            min: [180.0, 80.0, 0.0],
            max: [200.0, 100.0, 20.0],
        };
        let margin = 24.0;
        let viewport = [1440.0_f32, 900.0];
        for projection in both_projections() {
            let mut cam = Camera::new(viewport);
            cam.set_projection(projection);
            cam.set_angles(DEFAULT_AZIMUTH, DEFAULT_ELEVATION);
            cam.fit_boxes(&[near, far], margin);
            let mut lo = [f32::INFINITY; 2];
            let mut hi = [f32::NEG_INFINITY; 2];
            for bounds in [near, far] {
                for corner in bounds.corners() {
                    let s = cam.world_to_screen(corner);
                    for axis in 0..2 {
                        lo[axis] = lo[axis].min(s[axis]);
                        hi[axis] = hi[axis].max(s[axis]);
                    }
                }
            }
            for axis in 0..2 {
                let mid = (lo[axis] + hi[axis]) * 0.5;
                let centre = viewport[axis] * 0.5;
                assert!(
                    (mid - centre).abs() < 2.0,
                    "{projection:?} axis {axis}: the boxes' middle is {mid}, not {centre}"
                );
            }
            assert!(lo[0] >= margin - 0.5 && hi[0] <= viewport[0] - margin + 0.5);
            assert!(lo[1] >= margin - 0.5 && hi[1] <= viewport[1] - margin + 0.5);
        }
    }

    #[test]
    fn fit_boxes_room_leaves_the_extra_room_at_the_right_and_bottom() {
        let bounds = block();
        let margin = 24.0;
        let room = FitRoom {
            top: 60.0,
            right: 80.0,
            bottom: 120.0,
        };
        let viewport = [1440.0_f32, 900.0];
        for projection in both_projections() {
            let mut cam = Camera::new(viewport);
            cam.set_projection(projection);
            cam.set_angles(DEFAULT_AZIMUTH, DEFAULT_ELEVATION);
            cam.fit_boxes_room(&[bounds], margin, room);
            let mut lo = [f32::INFINITY; 2];
            let mut hi = [f32::NEG_INFINITY; 2];
            for corner in bounds.corners() {
                let s = cam.world_to_screen(corner);
                for axis in 0..2 {
                    lo[axis] = lo[axis].min(s[axis]);
                    hi[axis] = hi[axis].max(s[axis]);
                }
            }
            let mid_x = (lo[0] + hi[0]) * 0.5;
            let target_x = (viewport[0] - room.right) * 0.5;
            assert!(
                (mid_x - target_x).abs() < 2.0,
                "{projection:?}: the box's middle is {mid_x}, not {target_x}"
            );
            let mid_y = (lo[1] + hi[1]) * 0.5;
            let target_y = (viewport[1] + room.top - room.bottom) * 0.5;
            assert!(
                (mid_y - target_y).abs() < 2.0,
                "{projection:?}: the box's middle is {mid_y}, not {target_y}"
            );
            assert!(
                lo[1] >= margin + room.top - 0.5,
                "{projection:?}: the box keeps the extra room at the top"
            );
            assert!(
                hi[0] <= viewport[0] - margin - room.right + 0.5,
                "{projection:?}: the box keeps the extra room at the right"
            );
            assert!(
                hi[1] <= viewport[1] - margin - room.bottom + 0.5,
                "{projection:?}: the box keeps the extra room at the bottom"
            );
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

    #[test]
    fn centre_on_puts_a_point_in_the_air_on_the_screen_centre() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_zoom(12.0);
        let point = [30.0, -14.0, 80.0];
        cam.centre_on(point);
        let [x, y] = cam.world_to_screen(point);
        assert!((x - 720.0).abs() < 0.5, "x on the view axis: {x}");
        assert!(
            (y - cam.axis_y()).abs() < 0.5,
            "y on the view axis row {}: {y}",
            cam.axis_y()
        );
    }

    #[test]
    fn centring_on_a_point_in_the_air_keeps_it_one_eye_distance_away_at_the_zoom() {
        for elevation in [5.0, 20.0, 30.0, 60.0] {
            let mut cam = Camera::new([1440.0, 900.0]);
            cam.set_angles(0.0, elevation);
            cam.set_zoom(12.0);
            let point = [30.0, -14.0, 40.0];
            cam.centre_on(point);
            let eye = cam.eye();
            assert!(
                (distance(point, eye) - cam.eye_distance()).abs() < 1e-2,
                "the eye sits one eye distance from the centre at {elevation} degrees"
            );
            // A unit step along the screen's right vector moves the point by the zoom in pixels.
            let right = cam.basis().right;
            let projector = cam.projector();
            let at = projector.world_to_screen(point);
            let moved = projector.world_to_screen(std::array::from_fn(|i| point[i] + right[i]));
            assert!(
                (moved[0] - at[0] - 12.0).abs() < 0.05 && (moved[1] - at[1]).abs() < 0.05,
                "a unit at the centre spans the zoom at {elevation} degrees: {moved:?} from {at:?}"
            );
        }
    }

    /// A ground point `k` eye distances from the eye, straight down the view.
    fn along_view(cam: &Camera, k: f32) -> [f32; 3] {
        let (b, d, eye) = (cam.basis(), cam.eye_distance(), cam.eye());
        std::array::from_fn(|axis| eye[axis] - k * d * b.toward[axis])
    }

    #[test]
    fn fog_rises_from_zero_to_one_along_the_view() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_zoom(1.0);
        assert_eq!(
            cam.fog_at(along_view(&cam, 1.0)),
            0.0,
            "nothing fades at the pivot"
        );
        let end = theme::FOG_NEAR * theme::FOG_SPAN;
        assert_eq!(cam.fog_at(along_view(&cam, end)), 1.0);
        let mut last = 0.0;
        for step in 0..=200 {
            let value = cam.fog_at(along_view(&cam, step as f32 * 0.05));
            assert!((0.0..=1.0).contains(&value), "fog {value} at step {step}");
            assert!(value >= last - 1e-6, "fog fell at step {step}");
            last = value;
        }
    }

    #[test]
    fn the_fog_starts_past_the_far_corner_of_the_plant_at_low_tilt() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_fog_bounds(block());
        cam.set_angles(DEFAULT_AZIMUTH, 8.0);
        cam.fit(&block(), 24.0);
        for corner in block().corners() {
            assert_eq!(cam.fog_at(corner), 0.0, "corner {corner:?} is fogged");
        }
        let (start, end) = cam.fog_range();
        assert!(start > theme::FOG_NEAR * cam.eye_distance() * 0.99);
        assert!((end / start - theme::FOG_SPAN).abs() < 1e-4);
    }

    #[test]
    fn fog_is_measured_from_the_eye_so_the_pivot_is_clear_and_the_far_end_is_white() {
        let mut cam = Camera::new([1440.0, 900.0]);
        cam.set_zoom(1.0);
        let pivot = cam.pivot();
        assert_eq!(cam.fog_at([pivot[0], pivot[1], 0.0]), 0.0);
        let far_point = along_view(&cam, theme::FOG_NEAR * theme::FOG_SPAN);
        assert!(
            cam.fog_at(far_point) > 0.999,
            "a point past the fog's end is white"
        );
    }

    #[test]
    fn a_low_view_fades_the_far_ground_more_than_a_high_view() {
        let mut low = Camera::new([1440.0, 900.0]);
        low.set_zoom(1.0);
        let mut high = low;
        low.set_angles(DEFAULT_AZIMUTH, 10.0);
        high.set_angles(DEFAULT_AZIMUTH, 70.0);
        // Ground three eye distances behind the pivot, along the horizontal view direction.
        let behind = |c: &Camera| {
            let d = c.eye_distance();
            let (sin_a, cos_a) = c.azimuth().to_radians().sin_cos();
            let p = c.pivot();
            c.fog_at([p[0] - 3.0 * d * cos_a, p[1] - 3.0 * d * sin_a, 0.0])
        };
        assert!(
            behind(&low) > behind(&high),
            "low {} high {}",
            behind(&low),
            behind(&high)
        );
    }
}
