//! The Acan plant renderer. The scene is 3D meshes on an infinite grid, drawn by wgpu with a
//! perspective orbit camera (or orthographic on request), four-tone paper shading, feature
//! edges, and an object-ID outline pass. The model shape is the shared contract in
//! `ui-figure-contract-v2`.

pub mod camera;
pub mod edges;
pub mod geometry;
pub mod model;
pub mod renderer;
pub mod scene;
pub mod theme;

pub use camera::{Camera, Projection};
pub use model::{Aabb, Detail, Dot3, Ink, Line3, Mesh, Model, Surface};
pub use renderer::{RenderError, Renderer};
pub use scene::{Instance, Lod, Ribbon, Scene};
/// The wgpu version this crate was built against. Callers create the instance and surface with it.
pub use wgpu;
