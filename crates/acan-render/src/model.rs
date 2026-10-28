//! The model types are owned by acan-iso, which builds the models. The renderer re-exports them,
//! so its callers keep one import path and each type has one definition.

pub use acan_iso::{Aabb, Detail, Dot3, Ink, Line3, Mesh, Model, Surface};
