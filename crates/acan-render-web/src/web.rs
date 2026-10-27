//! Browser glue: the wgpu canvas, and the methods the page calls.
//!
//! The page feeds the origin's JSON in (the log, then each streamed event, and the plant), and
//! reads back the view as JSON for the HTML panels and the canvas position of each pin.

use acan_core::Stamped;
use acan_render::{Camera, Renderer, wgpu};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::layout::{Plan, PlantJson};
use crate::mirror::Mirror;
use crate::paint;

/// Space kept between the plan and the canvas edge when the camera fits it, in CSS pixels.
const FIT_MARGIN_PX: f32 = 56.0;
/// The page's paper color, `#f2f2ef`, as sRGB.
const PAPER: [f32; 4] = [0.949, 0.949, 0.937, 1.0];

/// The canvas, the camera, and the folded state the canvas draws.
#[wasm_bindgen]
pub struct App {
    renderer: Renderer,
    camera: Camera,
    mirror: Option<Mirror>,
    plan: Option<Plan>,
    view_json: String,
    view_version: u32,
    dirty: bool,
}

#[wasm_bindgen]
impl App {
    /// `backend` is `"webgpu"` or `"webgl"`. The canvas must already have its drawing-buffer size
    /// set to its CSS size times the device pixel ratio.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown backend or when no adapter can present to the canvas.
    pub async fn create(canvas: HtmlCanvasElement, backend: String) -> Result<App, JsError> {
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
        renderer.set_clear_color(PAPER);
        Ok(App {
            renderer,
            camera: Camera::new(css),
            mirror: None,
            plan: None,
            view_json: String::new(),
            view_version: 0,
            dirty: false,
        })
    }

    /// Backend and adapter in use, for example `"BrowserWebGpu / ..."`.
    pub fn backend(&self) -> String {
        let info = self.renderer.adapter_info();
        format!(
            "{:?} / {} / {:?}",
            info.backend,
            info.name,
            self.renderer.surface_format()
        )
    }

    /// Folds the log: a JSON array of stamped events from seq 0.
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON does not parse or the log does not replay.
    pub fn load_log(&mut self, json: &str) -> Result<(), JsError> {
        let log: Vec<Stamped> =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        self.mirror = Some(Mirror::from_log(log).map_err(|e| JsError::new(&e.to_string()))?);
        self.dirty = true;
        Ok(())
    }

    /// Applies one streamed event, given as its JSON.
    ///
    /// # Errors
    ///
    /// Returns an error when no log is loaded, the JSON does not parse, an event was skipped,
    /// or the event refers to something the log never created.
    pub fn ingest(&mut self, json: &str) -> Result<(), JsError> {
        let event: Stamped =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let mirror = self
            .mirror
            .as_mut()
            .ok_or_else(|| JsError::new("no log is loaded"))?;
        mirror
            .ingest(&event)
            .map_err(|e| JsError::new(&e.to_string()))?;
        self.dirty = true;
        Ok(())
    }

    /// The seq the next event will carry. The stream resumes from here.
    pub fn next_seq(&self) -> f64 {
        self.mirror
            .as_ref()
            .map_or(0.0, |mirror| mirror.next_seq().get() as f64)
    }

    /// Sets the plant from `GET /api/plant` and fits the camera to it.
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON does not parse.
    pub fn set_plant(&mut self, json: &str) -> Result<(), JsError> {
        let plant: PlantJson =
            serde_json::from_str(json).map_err(|e| JsError::new(&e.to_string()))?;
        let plan = Plan::new(&plant);
        if let Some((min, max)) = plan.bounds() {
            self.camera.fit(min, max, FIT_MARGIN_PX);
        }
        self.plan = Some(plan);
        self.dirty = true;
        Ok(())
    }

    /// The view as of the last frame, as JSON. Compare [`App::view_version`] to see it change.
    pub fn view_json(&self) -> String {
        self.view_json.clone()
    }

    /// Counts the view rebuilds, so the page redraws its panels only when the view changed.
    pub fn view_version(&self) -> u32 {
        self.view_version
    }

    /// Sets the canvas size, in CSS pixels, and its device pixel ratio.
    pub fn resize(&mut self, css_width: f32, css_height: f32, device_pixel_ratio: f32) {
        self.camera.viewport = [css_width, css_height];
        self.renderer.resize([
            (css_width * device_pixel_ratio).round() as u32,
            (css_height * device_pixel_ratio).round() as u32,
        ]);
    }

    /// Moves the view by a drag of `dx`, `dy` CSS pixels.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan_by_screen([dx, dy]);
    }

    /// Zooms by `factor` around the CSS-pixel point `x`, `y`, keeping it fixed on screen.
    pub fn zoom_at(&mut self, x: f32, y: f32, factor: f32) {
        self.camera.zoom_at([x, y], factor);
    }

    /// Fits the whole plan in the canvas.
    pub fn fit(&mut self) {
        if let Some((min, max)) = self.plan.as_ref().and_then(Plan::bounds) {
            self.camera.fit(min, max, FIT_MARGIN_PX);
        }
    }

    /// The current zoom, in CSS pixels per world unit.
    pub fn zoom(&self) -> f32 {
        self.camera.zoom
    }

    /// CSS-pixel position of a world point, relative to the canvas.
    pub fn world_to_screen(&self, x: f32, y: f32) -> Vec<f32> {
        self.camera.world_to_screen([x, y]).to_vec()
    }

    /// Draws one frame, rebuilding the scene first when the log or the plant changed.
    ///
    /// # Errors
    ///
    /// Returns an error when the surface is lost.
    pub fn render(&mut self) -> Result<bool, JsError> {
        if self.dirty {
            self.rebuild()?;
        }
        self.renderer
            .render(&self.camera)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("mirror", &self.mirror)
            .field("plan", &self.plan)
            .field("view_version", &self.view_version)
            .field("dirty", &self.dirty)
            .finish_non_exhaustive()
    }
}

impl App {
    fn rebuild(&mut self) -> Result<(), JsError> {
        let (Some(mirror), Some(plan)) = (&self.mirror, &self.plan) else {
            return Ok(());
        };
        let view = mirror.view(plan);
        self.renderer.set_scene(&paint::scene(plan, &view));
        self.view_json = serde_json::to_string(&view).map_err(|e| JsError::new(&e.to_string()))?;
        self.view_version += 1;
        self.dirty = false;
        Ok(())
    }
}
