//! Plant canvas renderer: instanced machine rects and triangulated belt
//! polylines on a flat plan view. The renderer takes a `wgpu::Surface` and
//! never learns what owns it, so a browser canvas and a native window are
//! interchangeable hosts.

mod camera;
mod renderer;
mod scene;

pub use camera::Camera;
pub use renderer::{RenderError, Renderer};
pub use scene::{BeltVertex, Polyline, Rect, Scene, tessellate_polyline};
pub use wgpu;
