//! Browser host for `acan-render`: mounts the renderer on a canvas and exposes
//! a small API to plain JavaScript. Only meaningful on wasm32.
#![cfg(target_arch = "wasm32")]

mod plant;

use acan_render::{Camera, Renderer, Scene, wgpu};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

const GRAPHITE: [f32; 4] = [0.106, 0.110, 0.118, 1.0];

#[wasm_bindgen]
pub struct App {
    renderer: Renderer,
    camera: Camera,
    scene: Scene,
}

#[wasm_bindgen]
impl App {
    /// `backend` is `"webgpu"` or `"webgl"`. The canvas must already have its
    /// drawing-buffer size set to `css size * devicePixelRatio`.
    pub async fn create(
        canvas: HtmlCanvasElement,
        backend: String,
        machines: u32,
        belts: u32,
    ) -> Result<App, JsError> {
        std::panic::set_hook(Box::new(|info| {
            web_sys::console::error_1(&info.to_string().into());
        }));
        let backends = match backend.as_str() {
            "webgpu" => wgpu::Backends::BROWSER_WEBGPU,
            "webgl" => wgpu::Backends::GL,
            other => return Err(JsError::new(&format!("unknown backend {other:?}"))),
        };
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let css = [canvas.client_width() as f32, canvas.client_height() as f32];
        let size = [canvas.width(), canvas.height()];
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| JsError::new(&e.to_string()))?;
        let mut renderer = Renderer::new(&instance, surface, size)
            .await
            .map_err(|e| JsError::new(&e.to_string()))?;
        renderer.set_clear_color(GRAPHITE);

        let scene = plant::generate(machines as usize, belts as usize);
        renderer.set_scene(&scene);
        let mut camera = Camera::new(css);
        if let Some((min, max)) = scene.bounds() {
            camera.fit(min, max, 24.0);
        }
        Ok(App { renderer, camera, scene })
    }

    /// Backend and adapter actually in use, for example `"BrowserWebGpu"` or `"Gl"`.
    pub fn backend(&self) -> String {
        let info = self.renderer.adapter_info();
        format!("{:?} / {} / {:?}", info.backend, info.name, self.renderer.surface_format())
    }

    pub fn resize(&mut self, css_width: f32, css_height: f32, device_pixel_ratio: f32) {
        self.camera.viewport = [css_width, css_height];
        self.renderer.resize([
            (css_width * device_pixel_ratio).round() as u32,
            (css_height * device_pixel_ratio).round() as u32,
        ]);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan_by_screen([dx, dy]);
    }

    pub fn zoom_at(&mut self, x: f32, y: f32, factor: f32) {
        self.camera.zoom_at([x, y], factor);
    }

    pub fn fit(&mut self) {
        if let Some((min, max)) = self.scene.bounds() {
            self.camera.fit(min, max, 24.0);
        }
    }

    pub fn zoom(&self) -> f32 {
        self.camera.zoom
    }

    /// Returns false when the browser skipped the frame.
    pub fn render(&mut self) -> Result<bool, JsError> {
        self.renderer.render(&self.camera).map_err(|e| JsError::new(&e.to_string()))
    }

    /// CSS-pixel position of a world point, relative to the canvas.
    pub fn world_to_screen(&self, x: f32, y: f32) -> Vec<f32> {
        self.camera.world_to_screen([x, y]).to_vec()
    }

    /// World-space `[min_x, min_y, width, height]` of machine `index`.
    pub fn machine_rect(&self, index: usize) -> Option<Vec<f32>> {
        self.scene.rects.get(index).map(|r| vec![r.min[0], r.min[1], r.size[0], r.size[1]])
    }

    pub fn machine_count(&self) -> usize {
        self.scene.rects.len()
    }
}
