//! The wasm surface the page drives. The camera works in device pixels, so the renderer's
//! stroke clamp is in real screen pixels. Positions that cross to JavaScript are CSS pixels.

use acan_render::{Camera, Lod, Renderer, project, wgpu};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::layout::{Layout, layout};
use crate::plant::{Plant, PlantMachine, from_scan_json, rest_labels, stress};
use crate::stand_in;

/// Space around the plant at fit, in CSS pixels.
const FIT_MARGIN_CSS: f32 = 24.0;

#[wasm_bindgen]
pub struct App {
    renderer: Renderer,
    camera: Camera,
    dpr: f32,
    plant: Option<Loaded>,
}

/// The plant on screen.
struct Loaded {
    repo: String,
    machines: Vec<PlantMachine>,
    layout: Layout,
    /// Projected bounds of the world: `[min x, min y, max x, max y]`.
    bounds: [f32; 4],
    /// Machines that carry a name at rest.
    rest: Vec<usize>,
}

impl Loaded {
    fn corners(&self) -> ([f32; 2], [f32; 2]) {
        let b = self.bounds;
        ([b[0], b[1]], [b[2], b[3]])
    }

    /// Projected point at fraction `(fx, fy)` of machine `index`'s footprint, if it exists.
    fn on_footprint(&self, index: usize, fx: f32, fy: f32) -> Option<[f32; 2]> {
        let placed = self.layout.machines.get(index)?;
        let (w, d) = placed.kind.footprint();
        Some(project(placed.x + w * fx, placed.y + d * fy, 0.0))
    }
}

#[wasm_bindgen]
impl App {
    /// Builds the renderer on `canvas`. `backend` is `"webgpu"` or `"webgl"`. Set the canvas
    /// drawing size before calling, then call `resize`.
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
        let size = [canvas.width().max(1), canvas.height().max(1)];
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(js_err)?;
        let mut renderer = Renderer::new(&instance, surface, size)
            .await
            .map_err(js_err)?;
        renderer.set_models(&stand_in::models()).map_err(js_err)?;
        Ok(App {
            renderer,
            camera: Camera::new([size[0] as f32, size[1] as f32]),
            dpr: 1.0,
            plant: None,
        })
    }

    /// Backend and adapter in use, for example `"BrowserWebGpu / ... / Bgra8Unorm"`.
    pub fn backend(&self) -> String {
        let info = self.renderer.adapter_info();
        format!(
            "{:?} / {} / {:?}",
            info.backend,
            info.name,
            self.renderer.surface_format()
        )
    }

    /// Loads the scan JSON that `GET /api/plant` returns, or a fixture of the same shape, and
    /// fits the view to it. Returns the machine count.
    pub fn set_plant(&mut self, json: &str) -> Result<u32, JsError> {
        let plant = from_scan_json(json).map_err(js_err)?;
        self.load(plant)
    }

    /// Loads a generated plant of `machines` machines for the benchmark.
    pub fn set_stress(&mut self, machines: u32) -> Result<u32, JsError> {
        self.load(stress(machines as usize))
    }

    /// The repository's folder name, or an empty string before a plant loads.
    pub fn repo_name(&self) -> String {
        self.plant
            .as_ref()
            .map_or_else(String::new, |loaded| loaded.repo.clone())
    }

    /// Call on every canvas size or device pixel ratio change. The canvas drawing size must
    /// already be `css size * device pixel ratio`, rounded the same way.
    pub fn resize(&mut self, css_width: f32, css_height: f32, device_pixel_ratio: f32) {
        self.dpr = device_pixel_ratio.max(0.25);
        let size = [
            (css_width * self.dpr).round().max(1.0) as u32,
            (css_height * self.dpr).round().max(1.0) as u32,
        ];
        self.camera.viewport = [size[0] as f32, size[1] as f32];
        self.renderer.resize(size);
    }

    /// Moves the view by a pointer drag of `(dx, dy)` CSS pixels.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan_by_screen([dx * self.dpr, dy * self.dpr]);
    }

    /// Zooms by `factor` around the CSS point `(x, y)`, which stays under the pointer.
    pub fn zoom_at(&mut self, x: f32, y: f32, factor: f32) {
        self.camera.zoom_at([x * self.dpr, y * self.dpr], factor);
    }

    /// Centres the plant and fits it to the viewport.
    pub fn fit(&mut self) {
        if let Some(loaded) = &self.plant {
            let (min, max) = loaded.corners();
            self.camera.fit(min, max, FIT_MARGIN_CSS * self.dpr);
        }
    }

    /// Centres machine `index` and sets the zoom to `ratio` times the fit zoom.
    pub fn focus(&mut self, index: u32, ratio: f32) {
        let Some(centre) = self
            .plant
            .as_ref()
            .and_then(|loaded| loaded.on_footprint(index as usize, 0.5, 0.5))
        else {
            return;
        };
        self.camera.center = centre;
        self.camera.zoom = ratio * self.fit_zoom();
    }

    /// Level of detail now: 0 far, 1 near, 2 close.
    pub fn detail(&self) -> u32 {
        self.lod() as u32
    }

    /// Draws one frame. Returns false when the browser skipped the frame.
    pub fn render(&mut self) -> Result<bool, JsError> {
        self.renderer
            .render(&self.camera, self.lod())
            .map_err(js_err)
    }

    /// Label positions as flat `[machine index, css x, css y]` triples. Labels sit under each
    /// plinth. `all` lists every machine, and otherwise only the ones named at rest.
    pub fn label_layout(&self, all: bool) -> Vec<f32> {
        let Some(loaded) = &self.plant else {
            return Vec::new();
        };
        let ids: Box<dyn Iterator<Item = usize> + '_> = if all {
            Box::new(0..loaded.machines.len())
        } else {
            Box::new(loaded.rest.iter().copied())
        };
        let mut out = Vec::new();
        for index in ids {
            let Some(anchor) = loaded.on_footprint(index, 0.5, 1.0) else {
                continue;
            };
            let screen = self.camera.world_to_screen(anchor);
            out.extend([index as f32, screen[0] / self.dpr, screen[1] / self.dpr]);
        }
        out
    }

    pub fn machine_name(&self, index: u32) -> String {
        self.plant
            .as_ref()
            .and_then(|loaded| loaded.machines.get(index as usize))
            .map_or_else(String::new, |machine| machine.name.clone())
    }

    pub fn machine_files(&self, index: u32) -> u32 {
        self.plant
            .as_ref()
            .and_then(|loaded| loaded.machines.get(index as usize))
            .map_or(0, |machine| machine.files as u32)
    }

    pub fn machine_count(&self) -> u32 {
        self.plant
            .as_ref()
            .map_or(0, |loaded| loaded.machines.len() as u32)
    }
}

impl App {
    fn load(&mut self, plant: Plant) -> Result<u32, JsError> {
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let scene = stand_in::scene(&laid);
        self.renderer.set_scene(&scene).map_err(js_err)?;
        let count = plant.machines.len() as u32;
        self.plant = Some(Loaded {
            repo: plant.repo,
            rest: rest_labels(&plant.machines),
            machines: plant.machines,
            layout: laid,
            bounds: scene.world.bounds,
        });
        self.fit();
        Ok(count)
    }

    fn fit_zoom(&self) -> f32 {
        match &self.plant {
            Some(loaded) => {
                let (min, max) = loaded.corners();
                self.camera.fit_zoom(min, max, FIT_MARGIN_CSS * self.dpr)
            }
            None => 1.0,
        }
    }

    fn lod(&self) -> Lod {
        Lod::for_ratio(self.camera.zoom / self.fit_zoom())
    }
}

fn js_err(err: impl std::fmt::Display) -> JsError {
    JsError::new(&err.to_string())
}
