//! Isometric machine models as draw lists.
//!
//! A model is a [`Figure`]: fills, lines and dots in painter order, already projected to the iso plane at
//! scale 1. The renderer draws it with its own theme and uses each item's [`Detail`] to decide what to show
//! at a given zoom. The geometry is ported from the anatomy kit (MIT, see `LICENSE-anatomy`).

mod camera;
mod figure;
mod geom;
mod models;
mod parts;
mod pen;
mod plan;
mod scene;
mod solid;

pub use figure::{Detail, Figure, Fill, Ink, Item};
pub use models::{Footprint, Kind, footprint, model};
