//! Plant canvas renderer. Figures arrive as draw lists in plan units, tessellated once
//! and drawn as GPU vector geometry with a camera uniform per frame. The renderer takes a
//! `wgpu::Surface` and never learns what owns it, so a browser canvas and a native window
//! are interchangeable hosts.

mod camera;
mod draw_list;
mod geometry;
mod projection;
mod renderer;
mod scene;
mod theme;

pub use camera::Camera;
pub use draw_list::{Detail, DrawList, Fill, Ink, Item};
pub use projection::{depth, project};
pub use renderer::{RenderError, Renderer};
pub use scene::{Instance, Lod, Scene};
pub use wgpu;
