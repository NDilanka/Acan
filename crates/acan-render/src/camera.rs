use bytemuck::{Pod, Zeroable};

/// Orthographic plan-view camera. World and screen both have y pointing down,
/// and screen units are CSS pixels so overlay labels can use them directly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub center: [f32; 2],
    /// Screen pixels per world unit.
    pub zoom: f32,
    pub viewport: [f32; 2],
}

impl Camera {
    pub const MIN_ZOOM: f32 = 0.02;
    pub const MAX_ZOOM: f32 = 50.0;

    pub fn new(viewport: [f32; 2]) -> Self {
        Self {
            center: [0.0, 0.0],
            zoom: 1.0,
            viewport,
        }
    }

    pub fn world_to_screen(&self, p: [f32; 2]) -> [f32; 2] {
        [
            (p[0] - self.center[0]) * self.zoom + self.viewport[0] * 0.5,
            (p[1] - self.center[1]) * self.zoom + self.viewport[1] * 0.5,
        ]
    }

    pub fn screen_to_world(&self, s: [f32; 2]) -> [f32; 2] {
        [
            (s[0] - self.viewport[0] * 0.5) / self.zoom + self.center[0],
            (s[1] - self.viewport[1] * 0.5) / self.zoom + self.center[1],
        ]
    }

    /// Moves the view so content follows a pointer drag of `delta` screen pixels.
    pub fn pan_by_screen(&mut self, delta: [f32; 2]) {
        self.center[0] -= delta[0] / self.zoom;
        self.center[1] -= delta[1] / self.zoom;
    }

    /// Zooms by `factor`, keeping the world point under `anchor` fixed on screen.
    pub fn zoom_at(&mut self, anchor: [f32; 2], factor: f32) {
        let before = self.screen_to_world(anchor);
        self.zoom = (self.zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        let after = self.screen_to_world(anchor);
        self.center[0] += before[0] - after[0];
        self.center[1] += before[1] - after[1];
    }

    /// Centers on the box `min..max` and zooms so it fills the viewport with a margin.
    pub fn fit(&mut self, min: [f32; 2], max: [f32; 2], margin_px: f32) {
        self.center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        let w = (max[0] - min[0]).max(1e-3);
        let h = (max[1] - min[1]).max(1e-3);
        let zx = (self.viewport[0] - 2.0 * margin_px).max(1.0) / w;
        let zy = (self.viewport[1] - 2.0 * margin_px).max(1.0) / h;
        self.zoom = zx.min(zy).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
    }

    pub(crate) fn uniform(&self) -> CameraUniform {
        CameraUniform {
            center: self.center,
            scale: [
                2.0 * self.zoom / self.viewport[0],
                -2.0 * self.zoom / self.viewport[1],
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct CameraUniform {
    pub center: [f32; 2],
    pub scale: [f32; 2],
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
    }

    #[test]
    fn screen_and_world_round_trip() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.center = [120.0, -40.0];
        cam.zoom = 2.5;
        let w = [33.0, 71.0];
        assert!(close(cam.screen_to_world(cam.world_to_screen(w)), w));
        assert!(close(cam.world_to_screen(cam.center), [400.0, 300.0]));
    }

    #[test]
    fn pan_moves_content_with_pointer() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.zoom = 4.0;
        let w = [10.0, 10.0];
        let before = cam.world_to_screen(w);
        cam.pan_by_screen([25.0, -12.0]);
        let after = cam.world_to_screen(w);
        assert!(close(after, [before[0] + 25.0, before[1] - 12.0]));
    }

    #[test]
    fn zoom_keeps_anchor_fixed() {
        let mut cam = Camera::new([800.0, 600.0]);
        let anchor = [610.0, 95.0];
        let w = cam.screen_to_world(anchor);
        cam.zoom_at(anchor, 3.0);
        assert!(close(cam.world_to_screen(w), anchor));
        assert!((cam.zoom - 3.0).abs() < 1e-6);
    }

    #[test]
    fn uniform_maps_viewport_corners_to_clip_corners() {
        let mut cam = Camera::new([800.0, 600.0]);
        cam.center = [5.0, 7.0];
        cam.zoom = 2.0;
        let u = cam.uniform();
        let top_left = cam.screen_to_world([0.0, 0.0]);
        let clip = [
            (top_left[0] - u.center[0]) * u.scale[0],
            (top_left[1] - u.center[1]) * u.scale[1],
        ];
        assert!(close(clip, [-1.0, 1.0]));
    }
}
