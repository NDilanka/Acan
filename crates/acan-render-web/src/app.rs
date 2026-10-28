//! The wasm surface the page drives. Pointer and key input arrive in CSS pixels, and the camera
//! works in device pixels, so this file scales input by the device pixel ratio. Positions that
//! cross back to JavaScript are CSS pixels.
//!
//! The plant is drawn in 3D from the scan JSON. The origin's log folds into a view on the next
//! frame after it changes, and the page reads that view back as JSON for its HTML panels.

use std::collections::HashMap;

use acan_core::Stamped;
use acan_render::camera::{DEFAULT_AZIMUTH, DEFAULT_ELEVATION};
use acan_render::{Aabb, Camera, Lod, Model, Renderer, wgpu};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::kind::PlanFootprint;
use crate::labels::{self, LabelSpot};
use crate::layout::{Layout, layout};
use crate::mirror::Mirror;
use crate::plan::{Plan, PlantJson};
use crate::plant::{Plant, PlantMachine, from_scan_json, rest_labels, stress};
use crate::scene;

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
    turn: Option<Turn>,
    plant: Option<Loaded>,
    plan: Option<Plan>,
    mirror: Option<Mirror>,
    view_json: String,
    view_version: u32,
    dirty: bool,
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
    /// The solid box of every machine, each with its machine index, for label occlusion.
    occluders: Vec<(usize, Aabb)>,
    /// Each machine's label half width in CSS pixels, estimated from its name.
    label_half_px: Vec<f32>,
    /// Whether each machine carries a name at rest once it is large enough on screen.
    rest: Vec<bool>,
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
        let models = scene::models();
        renderer.set_models(&models);
        Ok(App {
            renderer,
            camera: Camera::new([size[0] as f32, size[1] as f32]),
            models,
            dpr: 1.0,
            turn: None,
            plant: None,
            plan: None,
            mirror: None,
            view_json: String::new(),
            view_version: 0,
            dirty: false,
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

    /// Sets the plant from `GET /api/plant` and fits the view to it. Returns the machine count.
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON is not the plant's shape.
    pub fn set_plant(&mut self, json: &str) -> Result<u32, JsError> {
        let plant = from_scan_json(json).map_err(js_err)?;
        let plan_json: PlantJson = serde_json::from_str(json).map_err(js_err)?;
        Ok(self.load(plant, Some(&plan_json)))
    }

    /// Loads a generated plant of `machines` machines for the benchmark. It has no log, so the
    /// view stays empty.
    pub fn set_stress(&mut self, machines: u32) -> u32 {
        self.load(stress(machines as usize), None)
    }

    /// Folds the log: a JSON array of stamped events from seq 0.
    ///
    /// # Errors
    ///
    /// Returns an error when the JSON does not parse or the log does not replay.
    pub fn load_log(&mut self, json: &str) -> Result<(), JsError> {
        let log: Vec<Stamped> = serde_json::from_str(json).map_err(js_err)?;
        self.mirror = Some(Mirror::from_log(log).map_err(js_err)?);
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
        let event: Stamped = serde_json::from_str(json).map_err(js_err)?;
        let mirror = self
            .mirror
            .as_mut()
            .ok_or_else(|| js_err("no log is loaded"))?;
        mirror.ingest(&event).map_err(js_err)?;
        self.dirty = true;
        Ok(())
    }

    /// The seq the next event will carry. The stream resumes from here.
    pub fn next_seq(&self) -> f64 {
        self.mirror
            .as_ref()
            .map_or(0.0, |mirror| mirror.next_seq().get() as f64)
    }

    /// The view as of the last frame, as JSON. Compare [`App::view_version`] to see it change.
    pub fn view_json(&self) -> String {
        self.view_json.clone()
    }

    /// Counts the view rebuilds, so the page redraws its panels only when the view changed.
    pub fn view_version(&self) -> u32 {
        self.view_version
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

    /// Moves the scene so the ground under the CSS point `(x0, y0)` lands under `(x1, y1)`.
    /// Called with the previous and the current pointer position, the ground point stays under
    /// the pointer in either projection.
    pub fn drag(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.camera.drag_ground(
            [x0 * self.dpr, y0 * self.dpr],
            [x1 * self.dpr, y1 * self.dpr],
        );
    }

    /// Moves the scene by `(dx, dy)` CSS pixels as if dragged from the screen centre. The arrow
    /// keys and the benchmark use it.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let [vw, vh] = self.camera.viewport();
        let from = [vw * 0.5, vh * 0.5];
        self.camera
            .drag_ground(from, [from[0] + dx * self.dpr, from[1] + dy * self.dpr]);
    }

    /// Switches between the perspective and orthographic projections, keeping the framing.
    pub fn toggle_projection(&mut self) {
        self.camera.toggle_projection();
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

    /// Turns to the given angles in degrees and fits the plant there, centred on screen.
    pub fn view(&mut self, azimuth: f32, elevation: f32) {
        self.turn = None;
        self.camera.set_angles(azimuth, elevation);
        if let Some(loaded) = &self.plant {
            let bounds = loaded.bounds;
            self.camera.fit(&bounds, FIT_MARGIN_CSS * self.dpr);
        }
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

    /// Draws one frame, folding the log into the view first when it changed. Returns false when
    /// the browser skipped the frame.
    ///
    /// # Errors
    ///
    /// Returns an error when the surface is lost or the view cannot be encoded.
    pub fn render(&mut self) -> Result<bool, JsError> {
        if self.dirty {
            self.rebuild_view()?;
        }
        self.renderer
            .render(&self.camera, self.lod())
            .map_err(js_err)
    }

    /// Label candidates as flat `[machine index, css x, css y, opacity]` quads, most important
    /// first: the machines largest on screen lead, so the page keeps the larger or nearer one
    /// when two labels collide. Each label hangs under its plate's front edge, or above its
    /// machine when a nearer one hides that spot, and fades with the plant's fog. `all` names
    /// every machine large enough on screen. Otherwise the machines named at rest are, plus the
    /// largest few on screen, so a far plant still shows some names.
    pub fn label_layout(&self, all: bool) -> Vec<f32> {
        let Some(loaded) = &self.plant else {
            return Vec::new();
        };
        let named: Vec<bool> = if all {
            vec![true; loaded.machines.len()]
        } else {
            loaded.rest.clone()
        };
        let dpr = self.dpr;
        let rules = if all {
            labels::Rules {
                min_px: labels::ALL_MIN_CSS_PX * dpr,
                floor_px: labels::ALL_MIN_CSS_PX * dpr,
                budget: 0,
                drop_px: labels::LABEL_DROP_CSS_PX * dpr,
                lift_px: labels::LABEL_LIFT_CSS_PX * dpr,
            }
        } else {
            labels::Rules {
                min_px: labels::REST_MIN_CSS_PX * dpr,
                floor_px: labels::BUDGET_FLOOR_CSS_PX * dpr,
                budget: labels::REST_BUDGET,
                drop_px: labels::LABEL_DROP_CSS_PX * dpr,
                lift_px: labels::LABEL_LIFT_CSS_PX * dpr,
            }
        };
        let footprints: Vec<[f32; 4]> = loaded
            .layout
            .machines
            .iter()
            .map(|m| m.footprint())
            .collect();
        let half_widths: Vec<f32> = loaded.label_half_px.iter().map(|w| w * dpr).collect();
        let machines = labels::Machines {
            footprints: &footprints,
            occluders: &loaded.occluders,
            half_widths: &half_widths,
        };
        let spots: Vec<LabelSpot> = labels::place(&self.camera, &machines, &named, &rules);
        spots
            .iter()
            .flat_map(|spot| {
                [
                    spot.index as f32,
                    spot.screen[0] / self.dpr,
                    spot.screen[1] / self.dpr,
                    spot.opacity,
                ]
            })
            .collect()
    }

    /// CSS-pixel position of a plan point on the ground, relative to the canvas. The page pins
    /// its panels to the plant with it.
    pub fn world_to_screen(&self, x: f32, y: f32) -> Vec<f32> {
        let [px, py] = self.camera.world_to_screen([x, y, 0.0]);
        vec![px / self.dpr, py / self.dpr]
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
    /// Puts `plant` on screen. `plan_json` is the same plant's JSON when the page has one, and
    /// the plan places its machines where the layout puts them.
    fn load(&mut self, plant: Plant, plan_json: Option<&PlantJson>) -> u32 {
        let (kinds, edges) = plant.layout_input();
        let laid = layout(&kinds, &edges);
        let scene = scene::plant_scene(&laid, &self.models);
        self.renderer.set_scene(&scene);
        self.camera.set_fog_bounds(scene.bounds);
        let occluders = scene
            .instances
            .iter()
            .enumerate()
            .map(|(machine, inst)| {
                let model = self.models[usize::from(inst.kind)].bounds;
                let world = Aabb {
                    min: std::array::from_fn(|axis| inst.offset[axis] + model.min[axis]),
                    max: std::array::from_fn(|axis| inst.offset[axis] + model.max[axis]),
                };
                (machine, world)
            })
            .collect();
        let footprints: HashMap<&str, [f32; 4]> = plant
            .machines
            .iter()
            .zip(&laid.machines)
            .map(|(machine, placed)| (machine.name.as_str(), placed.footprint()))
            .collect();
        let count = plant.machines.len() as u32;
        self.plan = plan_json.map(|json| {
            Plan::new(json, |name| {
                footprints.get(name).copied().unwrap_or([0.0; 4])
            })
        });
        self.dirty = true;
        self.plant = Some(Loaded {
            repo: plant.repo,
            rest: {
                let mut named = vec![false; plant.machines.len()];
                for index in rest_labels(&plant.machines) {
                    named[index] = true;
                }
                named
            },
            label_half_px: plant
                .machines
                .iter()
                .map(|m| m.name.chars().count() as f32 * labels::CHAR_CSS_PX * 0.5)
                .collect(),
            machines: plant.machines,
            layout: laid,
            bounds: scene.bounds,
            occluders,
        });
        self.fit();
        count
    }

    fn rebuild_view(&mut self) -> Result<(), JsError> {
        self.dirty = false;
        let (Some(mirror), Some(plan)) = (&self.mirror, &self.plan) else {
            return Ok(());
        };
        let view = mirror.view(plan);
        self.view_json = serde_json::to_string(&view).map_err(js_err)?;
        self.view_version += 1;
        Ok(())
    }

    /// The zoom at which the plant fills the view at the default angles, in the current
    /// projection. Level of detail is measured against this, so orbiting never changes the
    /// detail.
    fn fit_zoom(&self) -> f32 {
        match &self.plant {
            Some(loaded) => {
                let mut probe = self.camera;
                probe.set_angles(DEFAULT_AZIMUTH, DEFAULT_ELEVATION);
                probe.fit(&loaded.bounds, FIT_MARGIN_CSS * self.dpr);
                probe.zoom()
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
