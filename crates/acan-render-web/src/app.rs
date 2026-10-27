//! The wasm surface the page drives. Pointer and key input arrive in CSS pixels, and the camera
//! works in device pixels, so this file scales input by the device pixel ratio. Positions that
//! cross back to JavaScript are CSS pixels.

use acan_render::camera::{DEFAULT_AZIMUTH, DEFAULT_ELEVATION, fit_at};
use acan_render::{Aabb, Camera, Lod, Model, Renderer, wgpu};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::layout::{Layout, layout};
use crate::plant::{Plant, PlantMachine, from_scan_json, rest_labels, stress};
use crate::stand_in;

/// Space around the plant at fit, in CSS pixels.
const FIT_MARGIN_CSS: f32 = 24.0;
/// Degrees of orbit per CSS pixel of drag.
const ORBIT_DEG_PER_PX: f32 = 0.4;
/// Degrees of tilt per CSS pixel of drag.
const TILT_DEG_PER_PX: f32 = 0.25;
/// Duration of a Q or E turn, in milliseconds.
const TURN_MS: f32 = 180.0;

#[wasm_bindgen]
pub struct App {
    renderer: Renderer,
    camera: Camera,
    models: Vec<Model>,
    dpr: f32,
    plant: Option<Loaded>,
    turn: Option<Turn>,
}

/// An ease from one azimuth to another, in degrees. `to` is not wrapped, so repeated turns add up.
struct Turn {
    from: f32,
    to: f32,
    elapsed_ms: f32,
}

/// The plant on screen.
struct Loaded {
    repo: String,
    machines: Vec<PlantMachine>,
    layout: Layout,
    /// Bounds of everything drawn, in plan units and height.
    bounds: Aabb,
    /// Machines that carry a name at rest.
    rest: Vec<usize>,
}

impl Loaded {
    /// The plinth corner nearest the viewer, on the ground, for a label to sit under.
    fn front_corner(&self, index: usize, toward: [f32; 2]) -> Option<[f32; 3]> {
        let placed = self.layout.machines.get(index)?;
        let (w, d) = placed.kind.footprint();
        let (cx, cy) = (placed.x + w / 2.0, placed.y + d / 2.0);
        let corner = [
            [cx - w / 2.0, cy - d / 2.0],
            [cx + w / 2.0, cy - d / 2.0],
            [cx + w / 2.0, cy + d / 2.0],
            [cx - w / 2.0, cy + d / 2.0],
        ]
        .into_iter()
        .max_by(|a, b| {
            let da = a[0] * toward[0] + a[1] * toward[1];
            let db = b[0] * toward[0] + b[1] * toward[1];
            da.total_cmp(&db)
        })?;
        Some([corner[0], corner[1], 0.0])
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
        let models = stand_in::models();
        renderer.set_models(&models);
        Ok(App {
            renderer,
            camera: Camera::new([size[0] as f32, size[1] as f32]),
            models,
            dpr: 1.0,
            plant: None,
            turn: None,
        })
    }

    /// Backend and adapter in use, with the scene sample count.
    pub fn backend(&self) -> String {
        let info = self.renderer.adapter_info();
        format!(
            "{:?} / {} / {:?} / msaa {}",
            info.backend,
            info.name,
            self.renderer.surface_format(),
            self.renderer.samples()
        )
    }

    /// Loads the scan JSON that `GET /api/plant` returns, or a fixture of the same shape, and
    /// fits the view to it. Returns the machine count.
    pub fn set_plant(&mut self, json: &str) -> Result<u32, JsError> {
        let plant = from_scan_json(json).map_err(js_err)?;
        Ok(self.load(plant))
    }

    /// Loads a generated plant of `machines` machines for the benchmark.
    pub fn set_stress(&mut self, machines: u32) -> u32 {
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
        self.camera.set_viewport([size[0] as f32, size[1] as f32]);
        self.renderer.resize(size);
    }

    /// Moves the ground under a drag of `(dx, dy)` CSS pixels, so the content follows the pointer.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan_px([dx * self.dpr, dy * self.dpr]);
    }

    /// Turns the view by `(dx, dy)` CSS pixels of drag: horizontal orbits, vertical tilts. This
    /// cancels a turn in progress, so the drag always wins.
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.turn = None;
        self.camera
            .orbit(dx * ORBIT_DEG_PER_PX, -dy * TILT_DEG_PER_PX);
    }

    /// Starts an ease of `degrees` of azimuth. Repeated calls add up.
    pub fn turn(&mut self, degrees: f32) {
        let (from, to) = match self.turn.take() {
            Some(turn) => (self.camera.azimuth(), turn.to + degrees),
            None => (self.camera.azimuth(), self.camera.azimuth() + degrees),
        };
        self.turn = Some(Turn {
            from,
            to,
            elapsed_ms: 0.0,
        });
    }

    /// Moves an active turn on by `dt_ms`. Returns true while it is still moving, so the page
    /// asks for another frame only then.
    pub fn advance(&mut self, dt_ms: f32) -> bool {
        let Some(turn) = self.turn.as_mut() else {
            return false;
        };
        turn.elapsed_ms += dt_ms.max(0.0);
        let t = (turn.elapsed_ms / TURN_MS).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        let azimuth = turn.from + (turn.to - turn.from) * eased;
        let elevation = self.camera.elevation();
        self.camera.set_angles(azimuth, elevation);
        if t >= 1.0 {
            self.turn = None;
            return false;
        }
        true
    }

    /// Zooms by `factor` around the CSS point `(x, y)`, which stays under the pointer.
    pub fn zoom_at(&mut self, x: f32, y: f32, factor: f32) {
        self.camera.zoom_at([x * self.dpr, y * self.dpr], factor);
    }

    /// Returns to the default angles and fits the plant.
    pub fn fit(&mut self) {
        self.turn = None;
        if let Some(loaded) = &self.plant {
            let bounds = loaded.bounds;
            self.camera.reset_view(&bounds, FIT_MARGIN_CSS * self.dpr);
        }
    }

    /// Centres machine `index` and sets the zoom to `ratio` times the fit zoom.
    pub fn focus(&mut self, index: u32, ratio: f32) {
        let Some(loaded) = self.plant.as_ref() else {
            return;
        };
        let Some(placed) = loaded.layout.machines.get(index as usize) else {
            return;
        };
        let (w, d) = placed.kind.footprint();
        self.camera
            .set_pivot([placed.x + w / 2.0, placed.y + d / 2.0]);
        self.camera.set_zoom(ratio * self.fit_zoom());
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
    /// plinth's front corner, projected in the same frame as the render. `all` lists every
    /// machine, and otherwise only the ones named at rest.
    pub fn label_layout(&self, all: bool) -> Vec<f32> {
        let Some(loaded) = &self.plant else {
            return Vec::new();
        };
        let toward = self.camera.basis().toward;
        let toward = [toward[0], toward[1]];
        let count = if all {
            loaded.machines.len()
        } else {
            loaded.rest.len()
        };
        let mut out = Vec::new();
        for slot in 0..count {
            let index = if all { slot } else { loaded.rest[slot] };
            let Some(anchor) = loaded.front_corner(index, toward) else {
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
    fn load(&mut self, plant: Plant) -> u32 {
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let scene = stand_in::scene(&laid, &self.models);
        self.renderer.set_scene(&scene);
        let count = plant.machines.len() as u32;
        self.plant = Some(Loaded {
            repo: plant.repo,
            rest: rest_labels(&plant.machines),
            machines: plant.machines,
            layout: laid,
            bounds: scene.bounds,
        });
        self.fit();
        count
    }

    /// The zoom at which the plant fills the view at the default angles. Level of detail is
    /// measured against this, so orbiting never changes the detail.
    fn fit_zoom(&self) -> f32 {
        match &self.plant {
            Some(loaded) => {
                fit_at(
                    self.camera.viewport(),
                    &loaded.bounds,
                    FIT_MARGIN_CSS * self.dpr,
                    DEFAULT_AZIMUTH,
                    DEFAULT_ELEVATION,
                )
                .1
            }
            None => 1.0,
        }
    }

    fn lod(&self) -> Lod {
        Lod::for_ratio(self.camera.zoom() / self.fit_zoom())
    }
}

fn js_err(err: impl std::fmt::Display) -> JsError {
    JsError::new(&err.to_string())
}
